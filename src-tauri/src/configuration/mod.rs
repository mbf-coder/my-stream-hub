//! Persistent app configuration (Phase 1).
//!
//! JSON file in the app-data directory. Secrets are NEVER stored here —
//! stream keys live in the OS keyring ([`crate::security`]).

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Full settings surface (Phase 7): general + video + audio + stream/recording
/// defaults. Secrets are never part of this struct (see [`crate::security`]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub version: u32,
    pub language: String,
    pub theme: Theme,
    pub base_width: u32,
    pub base_height: u32,
    pub output_width: u32,
    pub output_height: u32,
    pub fps: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub stream: StreamDefaults,
    pub recording: RecordingDefaults,
    pub transition_kind: String,
    pub transition_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamDefaults {
    pub encoder: String,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordingDefaults {
    pub format: String,
    pub directory: String,
    pub video_bitrate_kbps: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            language: "it".into(),
            theme: Theme::Dark,
            base_width: 1920,
            base_height: 1080,
            output_width: 1920,
            output_height: 1080,
            fps: 60,
            sample_rate: 48000,
            channels: 2,
            stream: StreamDefaults {
                encoder: "auto".into(),
                video_bitrate_kbps: 6000,
                audio_bitrate_kbps: 160,
            },
            recording: RecordingDefaults {
                format: "mkv".into(),
                directory: "recordings".into(),
                video_bitrate_kbps: 8000,
            },
            transition_kind: "cut".into(),
            transition_ms: 300,
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> EngineResult<()> {
        if self.language.len() > 16 || self.language.trim().is_empty() {
            return Err(EngineError::InvalidParameter(
                "language".into(),
                "must be 1..=16 chars".into(),
            ));
        }
        for (label, w, h) in [
            ("base", self.base_width, self.base_height),
            ("output", self.output_width, self.output_height),
        ] {
            if !(160..=7680).contains(&w) || !(90..=4320).contains(&h) {
                return Err(EngineError::InvalidParameter(
                    label.into(),
                    "resolution must be 160..=7680 x 90..=4320".into(),
                ));
            }
        }
        if !(1..=120).contains(&self.fps) {
            return Err(EngineError::InvalidParameter(
                "fps".into(),
                "fps must be 1..=120".into(),
            ));
        }
        if ![44100, 48000, 96000].contains(&self.sample_rate) {
            return Err(EngineError::InvalidParameter(
                "sample_rate".into(),
                "must be 44100/48000/96000".into(),
            ));
        }
        if !(1..=8).contains(&self.channels) {
            return Err(EngineError::InvalidParameter(
                "channels".into(),
                "must be 1..=8".into(),
            ));
        }
        if self.stream.encoder.trim().is_empty() {
            return Err(EngineError::InvalidParameter(
                "stream.encoder".into(),
                "must not be empty".into(),
            ));
        }
        if !matches!(self.recording.format.as_str(), "mkv" | "mp4") {
            return Err(EngineError::InvalidParameter(
                "recording.format".into(),
                "must be mkv or mp4".into(),
            ));
        }
        if !matches!(self.transition_kind.as_str(), "cut" | "fade") {
            return Err(EngineError::InvalidParameter(
                "transition_kind".into(),
                "must be cut or fade".into(),
            ));
        }
        Ok(())
    }

    /// Serialise to a JSON string (for the settings file).
    pub fn to_json(&self) -> EngineResult<String> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|e| EngineError::Internal(format!("serialise settings: {e}")))
    }

    /// Parse settings JSON (unknown future fields are ignored).
    pub fn from_json(json: &str) -> EngineResult<Self> {
        let cfg: Self = serde_json::from_str(json)
            .map_err(|e| EngineError::InvalidParameter("settings".into(), e.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn file_name() -> &'static str {
        "mystreamhub.config.json"
    }
}

/// Resolve `<app_data>/mystreamhub.config.json` without Tauri (pure path join
/// so it is unit-testable); the Tauri path resolver is used in `run_app`.
pub fn config_path_for_dir(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join(AppConfig::file_name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        assert!(AppConfig::default().validate().is_ok());
    }

    #[test]
    fn rejects_absurd_fps() {
        let c = AppConfig {
            fps: 0,
            ..AppConfig::default()
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn settings_json_round_trip() {
        let cfg = AppConfig::default();
        let json = cfg.to_json().unwrap();
        let back = AppConfig::from_json(&json).unwrap();
        assert_eq!(cfg, back);
        assert!(AppConfig::from_json("{ not json").is_err());
        assert!(AppConfig::from_json(r#"{"version":1,"language":"it","theme":"dark","base_width":1,"base_height":1080,"output_width":1920,"output_height":1080,"fps":60,"sample_rate":48000,"channels":2,"stream":{"encoder":"auto","video_bitrate_kbps":6000,"audio_bitrate_kbps":160},"recording":{"format":"mkv","directory":"recordings","video_bitrate_kbps":8000},"transition_kind":"cut","transition_ms":300}"#).is_err());
    }

    #[test]
    fn serialises_without_secrets() {
        let json = serde_json::to_string(&AppConfig::default()).unwrap();
        assert!(!json.to_lowercase().contains("key"));
        assert!(!json.to_lowercase().contains("token"));
        assert!(!json.to_lowercase().contains("password"));
    }
}
