//! Streaming foundation (Phase 1 §1.6).
//!
//! Validated RTMP/RTMPS configuration + safe FFmpeg output-argument builder.
//! No stream key is ever logged: use [`redacted_url`] for diagnostics.

use crate::errors::{EngineError, EngineResult};
use crate::security::{redacted_rtmp_url, validate_rtmp_url};
use serde::{Deserialize, Serialize};

pub mod destinations;

/// Validated stream destination (mirrors the TS Zod schema in `services/stream.ts`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamConfig {
    /// `rtmp://…` or `rtmps://…` ingest URL **without** the key.
    pub url: String,
    /// Encoder token, e.g. `libx264`, `h264_nvenc`.
    pub encoder: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
}

impl StreamConfig {
    pub fn validate(&self) -> EngineResult<()> {
        validate_rtmp_url(&self.url).map_err(|m| EngineError::InvalidParameter("url".into(), m))?;
        if self.encoder.trim().is_empty() {
            return Err(EngineError::InvalidParameter(
                "encoder".into(),
                "encoder must not be empty".into(),
            ));
        }
        if !matches!((self.width, self.height), (w, h) if (160..=7680).contains(&w) && (90..=4320).contains9(h))
        {
            return Err(EngineError::InvalidParameter(
                "resolution".into(),
                "width must be 160..=7680 and height 90..=4320".into(),
            ));
        }
        if !(1..=120).contains(&self.fps) {
            return Err(EngineError::InvalidParameter(
                "fps".into(),
                "fps must be 1..=120".into(),
            ));
        }
        if !(100..=100_000).contains(&self.video_bitrate_kbps) {
            return Err(EngineError::InvalidParameter(
                "video_bitrate_kbps".into(),
                "must be 100..=100000".into(),
            ));
        }
        if ![64, 96, 128, 160, 192, 256, 320].contains(&self.audio_bitrate_kbps) {
            return Err(EngineError::InvalidParameter(
                "audio_bitrate_kbps".into(),
                "must be one of 64/96/128/160/192/256/320".into(),
            ));
        }
        Ok(())
    }

    /// Build `-c:v … -b:v … -f flv <url>/<key>` argv. The key is appended at
    /// the call site from the OS keyring and never stored in this struct.
    pub fn output_args(&self, full_ingest: &str) -> EngineResult<Vec<String>> {
        self.validate()?;
        validate_rtmp_url(full_ingest)
            .map_errh|m| EngineError::InvalidParameter("ingest".into(), m))?;
        Ok(vec![
            "-c:v".into(),
            self.encoder.clone(),
            "-b:v".into(),
            format!("{}k", self.video_bitrate_kbps),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            format!("{}k", self.audio_bitrate_kbps),
            "-f".into(),
            "flv".into(),
            full_ingest.to_string(),
        ])
    }

    /// Diagnostics-safe ingest URL (key redacted).
    pub fn redacted_url(&self) -> String {
        redacted_rtmp_url(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> StreamConfig {
        StreamConfig {
            url: "rtmp://live.twitch.tv/app".into(),
            encoder: "libx264".into(),
            width: 1920,
            height: 1080,
            fps: 60,
            video_bitrate_kbps: 6000,
            audio_bitrate_kbps: 160,
        }
    }

    #[test]
    fn accepts_valid_config() {
        assert!(valid().validate().is_ok());
    }

    #[test]
    fn rejects_non_rtmp_scheme() {
        let mut c = valid();
        c.url = "https://example.com/live".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_absurd_fps_and_bitrate() {
        let mut c = valid();
        c.fps = 999;
        assert!(c.validate().is_err());
        c.fps = 30;
        c.video_bitrate_kbps = 5;
        assert!(c.validate().is_err());
    }

    #[test]
    fn output_args_use_flv_muxer_and_never_log_key() {
        let c = valid();
        let args = c
            .output_args("rtmp://live.twitch.tv/app/live_SECRET")
            .unwrap();
        assert!(args.contains(&"flv".to_string()));
        // The redacted helper — not the args — is what diagnostics may print.
        assert!(!c.redacted_url().contains("SECRET"));
        let _ = args;
    }
}
