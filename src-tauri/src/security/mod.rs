//! Security: validation, URL redaction, OS credential store (Phase 1 §3.4).
//!
//! Rules enforced here:
//! - stream keys never appear in logs, errors or IPC payloads;
//! - RTMP URLs are allow-listed by scheme (`rtmp`/`rtmps` only);
//! - file paths must stay inside an allowed base directory;
//! - secrets at rest go to the OS credential manager (Windows Credential
//!   Manager via `keyring`), never to config files or logs.

use crate::errors::{EngineError, EngineResult};
use std::path::{Path, PathBuf};

/// Credential-store service namespace (single source of truth).
pub const KEYRING_SERVICE: &str = "com.mystreamhub.studio";
/// Keyring account holding the active stream key.
pub const STREAM_KEY_ACCOUNT: &str = "active-stream-key";

/// Accept only `rtmp://…` and `rtmps://…` ingest URLs.
pub fn validate_rtmp_url(url: &str) -> Result<(), String> {
    if url.len() > 2048 {
        return Err("url too long".into());
    }
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("rtmp://") || lower.starts_with("rtmps://")) {
        return Err("scheme must be rtmp:// or rtmps://".into());
    }
    if url.contains(char::is_whitespace) {
        return Err("url must not contain whitespace".into());
    }
    Ok(())
}

/// Replace the trailing path segment (the stream key) with `***`.
/// `rtmp://host/app/live_abc123` → `rtmp://host/app/***`.
pub fn redacted_rtmp_url(url: &str) -> String {
    match url.rfind('/') {
        Some(i) if i + 1 < url.len() => format!("{}/***", &url[..i]),
        _ => "***".to_string(),
    }
}

/// Ensure `candidate` resolves inside `base` (no `..` escapes, absolute paths
/// must already be contained). Used for recording paths and stinger assets.
pub fn validate_path_inside_base(base: &Path, candidate: &str) -> EngineResult<PathBuf> {
    if candidate.len() > 4096 {
        return Err(EngineError::InvalidParameter(
            "path".into(),
            "path too long".into(),
        ));
    }
    let joined = base.join(candidate);
    // Lexical containment check (no I/O, no symlink resolution for Phase 1):
    // normalise `.`/`..` lexically, then require the result to stay inside `base`.
    fn normalise(p: &Path) -> PathBuf {
        use std::path::Component::*;
        let mut out = PathBuf::new();
        for comp in p.components() {
            match comp {
                Prefix(p) => out.push(p.as_os_str()),
                RootDir => out.push("/"),
                CurDir => {}
                ParentDir => {
                    out.pop();
                }
                Normal(c) => out.push(c),
            }
        }
        out
    }
    if !normalise(&joined).starts_with(normalise(base)) {
        return Err(EngineError::InvalidParameter(
            "path".into(),
            "path escapes the allowed directory".into(),
        ));
    }
    Ok(joined)
}

/// Store the active stream key in the OS credential manager.
/// Compiled against the real `keyring` backend in app builds; in
/// `--no-default-features` verification builds this returns an explanatory
/// error so CI stays hermetic without a secret-service daemon.
pub fn store_stream_key(key: &str) -> EngineResult<()> {
    if key.trim().is_empty() || key.len() > 4096 {
        return Err(EngineError::InvalidParameter(
            "stream_key".into(),
            "stream key must be 1..=4096 chars".into(),
        ));
    }
    #[cfg(feature = "tauri-app")]
    {
        let entry = keyring::Entry::new(KEYRING_SERVICE, STREAM_KEY_ACCOUNT)
            .map_err(|_| EngineError::CredentialStore)?;
        entry
            .set_password(key)
            .map_err(|_| EngineError::CredentialStore)?;
        Ok(())
    }
    #[cfg(not(feature = "tauri-app"))]
    {
        let _ = key;
        Err(EngineError::CredentialStore)
    }
}

/// Retrieve the active stream key (same gating as [`store_stream_key`]).
pub fn load_stream_key() -> EngineResult<String> {
    #[cfg(feature = "tauri-app")]
    {
        let entry = keyring::Entry::new(KEYRING_SERVICE, STREAM_KEY_ACCOUNT)
            .map_err(|_| EngineError::CredentialStore)?;
        entry
            .get_password()
            .map_err(|_| EngineError::CredentialStore)
    }
    #[cfg(not(feature = "tauri-app"))]
    {
        Err(EngineError::CredentialStore)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_rtmp_and_rtmps() {
        assert!(validate_rtmp_url("rtmp://live.twitch.tv/app").is_ok());
        assert!(validate_rtmp_url("rtmps://example.com:443/live").is_ok());
    }

    #[test]
    fn rejects_dangerous_schemes() {
        assert!(validate_rtmp_url("https://example.com/live").is_err());
        assert!(validate_rtmp_url("rtmp://exa mple.com/live").is_err());
        assert!(validate_rtmp_url("rtmp://x; do-evil").is_err());
    }

    #[test]
    fn redaction_hides_key_segment() {
        assert_eq!(
            redacted_rtmp_url("rtmp://live.twitch.tv/app/live_abc123"),
            "rtmp://live.twitch.tv/app/***"
        );
        assert!(!redacted_rtmp_url("rtmp://h/app/SECRET").contains("SECRET"));
    }

    #[test]
    fn path_escape_is_rejected() {
        let base = Path::new("/tmp/hub");
        assert!(validate_path_inside_base(base, "recordings/out.mkv").is_ok());
        assert!(validate_path_inside_base(base, "recordings/../..//escape.mkv").is_err());
    }

    #[test]
    fn empty_stream_key_is_rejected_before_touching_os_store() {
        assert!(store_stream_key("").is_err());
        assert!(store_stream_key("   ").is_err());
    }
}
