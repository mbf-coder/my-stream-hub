//! Security: RTMP allow-list, URL redaction, path confinement, OS keyring.

use crate::errors::{EngineError, EngineResult};
use std::path::{Path, PathBuf};

pub const KEYRING_SERVICE: &str = "com.mystreamhub.studio";
pub const STREAM_KEY_ACCOUNT: &str = "active-stream-key";

pub fn validate_rtmp_url(url: &str) -> Result<(), String> {
    if url.len() > 2048 { return Err("url too long".into()); }
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("rtmp://") || lower.starts_with("rtmps://")) { return Err("scheme must be rtmp:// or rtmps://".into()); }
    if url.contains(char::is_whitespace) { return Err("url must not contain whitespace".into()); }
    Ok(())
}

pub fn redacted_rtmp_url(url: &str) -> String {
    match url.rfind('/') {
        Some(i) if i + 1 < url.len() => format!("{}/***", &url[..i]),
        _ => "***".to_string(),
    }
}

pub fn validate_path_inside_base(base: &Path, candidate: &str) -> EngineResult<PathBuf> {
    if candidate.len() > 4096 { return Err(EngineError::InvalidParameter("path".into(), "path too long".into())); }
    let joined = base.join(candidate);
    fn normalise(p: &Path) -> PathBuf {
        use std::path::Component::*;
        let mut out = PathBuf::new();
        for comp in p.components() {
            match comp {
                Prefix(p) => out.push(p.as_os_str()),
                RootDir => out.push("/"),
                CurDir => {}
                ParentDir => { out.pop(); }
                Normal(c) => out.push(c),
            }
        }
        out
    }
    if !normalise(&joined).starts_with(normalise(base)) {
        return Err(EngineError::InvalidParameter("path".into(), "path escapes the allowed directory".into()));
    }
    Ok(joined)
}

pub fn store_stream_key(key: &str) -> EngineResult<()> {
    if key.trim().is_empty() || key.len() > 4096 {
        return Err(EngineError::InvalidParameter("stream_key".into(), "stream key must be 1..=4096 chars".into()));
    }
    #[cfg(feature = "tauri-app")]
    {
        let entry = keyring::Entry::new(KEYRING_SERVICE, STREAM_KEY_ACCOUNT).map_err(|_| EngineError::CredentialStore)?;
        entry.set_password(key).map_err(|_| EngineError::CredentialStore)?;
        Ok(())
    }
    #[cfg(not(feature = "tauri-app"))]
    {
        let _ = key;
        Err(EngineError::CredentialStore)
    }
}

pub fn load_stream_key() -> EngineResult<String> {
    #[cfg(feature = "tauri-app")]
    {
        let entry = keyring::Entry::new(KEYRING_SERVICE, STREAM_KEY_ACCOUNT).map_err(|_| EngineError::CredentialStore)?;
        entry.get_password().map_err(|_| EngineError::CredentialStore)
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
    }
    #[test]
    fn redaction_hides_key_segment() {
        assert_eq!(redacted_rtmp_url("rtmp://live.twitch.tv/app/live_abc123"), "rtmp://live.twitch.tv/app/***");
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
