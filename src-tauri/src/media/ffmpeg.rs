//! FFmpeg / FFprobe discovery (Phase 1).
//!
//! Shells out to `ffmpeg -version`, `ffprobe -version`, `ffmpeg -encoders`.
//! All parsing is pure and unit-tested with fixtures (CI without FFmpeg).

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::process::Command;

pub const PROBE_TIMEOUT_MS: u64 = 5_000;
pub const FFMPEG_BIN: &str = "ffmpeg";
pub const FFPROBE_BIN: &str = "ffprobe";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MediaCapabilities {
    pub ffmpeg_version: Option<String>,
    pub ffprobe_version: Option<String>,
    pub has_libx264: bool,
    pub has_aac: bool,
    pub rtmp_supported: bool,
}

pub async fn probe_version(binary: &str) -> EngineResult<String> {
    let out = tokio::time::timeout(Duration::from_millis(PROBE_TIMEOUT_MS), Command::new(binary).arg("-version").output())
        .await.map_err(|_| EngineError::Timeout(PROBE_TIMEOUT_MS))?
        .map_err(|e| if binary == FFMPEG_BIN { EngineError::FfmpegUnavailable(e.to_string()) } else { EngineError::FfprobeUnavailable(e.to_string()) })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(if binary == FFMPEG_BIN { EngineError::FfmpegUnavailable(stderr.trim().to_string()) } else { EngineError::FfprobeUnavailable(stderr.trim().to_string()) });
    }
    parse_version(&String::from_utf8_lossy(&out.stdout)).ok_or_else(|| if binary == FFMPEG_BIN { EngineError::FfmpegUnavailable("unparseable -version output".into()) } else { EngineError::FfprobeUnavailable("unparseable -version output".into()) })
}

/// Parse `ffmpeg version 7.1.1-... Copyright ...` → `7.1.1-...`.
pub fn parse_version(first_line_output: &str) -> Option<String> {
    let first = first_line_output.lines().next()?;
    let mut it = first.split_whitespace();
    let _name = it.next()?;
    if it.next()? != "version" { return None; }
    it.next().map(|v| v.to_string())
}

pub fn capabilities_from_encoders(ffmpeg_version: Option<String>, ffprobe_version: Option<String>, encoders_output: &str) -> MediaCapabilities {
    let lower = encoders_output.to_lowercase();
    MediaCapabilities {
        ffmpeg_version, ffprobe_version,
        has_libx264: lower.contains("libx264"),
        has_aac: lower.contains(" aac "),
        rtmp_supported: !encoders_output.trim().is_empty(),
    }
}

pub async fn fetch_encoders_output() -> EngineResult<String> {
    let out = tokio::time::timeout(Duration::from_millis(PROBE_TIMEOUT_MS), Command::new(FFMPEG_BIN).args(["-hide_banner", "-encoders"]).output())
        .await.map_err(|_| EngineError::Timeout(PROBE_TIMEOUT_MS))?
        .map_err(|e| EngineError::FfmpegUnavailable(e.to_string()))?;
    if !out.status.success() {
        return Err(EngineError::FfmpegUnavailable(String::from_utf8_lossy(&out.stderr).trim().to_string()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_stock_version_lines() {
        assert_eq!(parse_version("ffmpeg version 7.1.1-essentials_build Copyright (c)"), Some("7.1.1-essentials_build".into()));
        assert_eq!(parse_version("garbage line"), None);
        assert_eq!(parse_version(""), None);
    }
    #[test]
    fn derives_capabilities_from_encoder_list() {
        let caps = capabilities_from_encoders(Some("7.1".into()), Some("7.1".into()), "Encoders:\\n V..... libx264   x264\\n A..... aac   AAC\\n");
        assert!(caps.has_libx264 && caps.has_aac && caps.rtmp_supported);
    }
    #[test]
    fn empty_encoder_list_disables_rtmp_assumption() {
        let caps = capabilities_from_encoders(None, None, "   \\n");
        assert!(!caps.has_libx264 && !caps.rtmp_supported);
    }
}
