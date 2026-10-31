//! Local recording — Phase 6.
//!
//! MKV / MP4 outputs from the program feed with selectable encoder, plus a
//! `remux_mkv_to_mp4` helper (stream-copy, no re-encode). Paths are confined
//! to the recordings base directory. Recording shares the engine's single
//! pipeline slot with the media test and streaming (documented in the UI).

use crate::errors::{EngineError, EngineResult};
use crate::security::validate_path_inside_base;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Container format for local recordings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecordingFormat {
    #[default]
    Mkv,
    Mp4,
}

impl RecordingFormat {
    pub fn extension(self) -> &'static str {
        match self {
            RecordingFormat::Mkv => "mkv",
            RecordingFormat::Mp4 => "mp4",
        }
    }
    pub fn muxer(self) -> &'static str {
        match self {
            RecordingFormat::Mkv => "matroska",
            RecordingFormat::Mp4 => "mp4",
        }
    }
}

/// What to capture: separate audio/video tracks or a single mixed track.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrackMode {
    #[default]
    Mixed,
    Separate,
}

/// Recording request (validated before any spawn).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordingConfig {
    /// File name relative to the recordings base dir, e.g. `show-01.mkv`.
    pub file_name: String,
    pub format: RecordingFormat,
    pub tracks: TrackMode,
    /// Encoder token, e.g. `libx264`.
    pub encoder: String,
    pub video_bitrate_kbps: u32,
}

impl RecordingConfig {
    pub fn validate(&self, base_dir: &Path) -> EngineResult<PathBuf> {
        if self.encoder.trim().is_empty() {
            return Err(EngineError::InvalidParameter(
                "encoder".into(),
                "encoder must not be empty".into(),
            ));
        }
        if !(100..=100_000).contains(&self.video_bitrate_kbps) {
            return Err(EngineError::InvalidParameter(
                "video_bitrate_kbps".into(),
                "must be 100..=100000".into(),
            ));
        }
        let path = validate_path_inside_base(base_dir, &self.file_name)?;
        let expected = self.format.extension();
        let actual = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if actual.to_ascii_lowercase() != expected {
            return Err(EngineError::InvalidParameter(
                "file_name".into(),
                format!("extension must be .{expected} for this format"),
            ));
        }
        Ok(path)
    }

    /// FFmpeg output-side argv for recording. `input_args` come from the caller
    /// (test pattern or `dshow` device) so this builder stays input-agnostic.
    pub fn build_args(&self, out_path: &Path, encoder: &str, input_args: &[String]) -> Vec<String> {
        let mut args = vec![
            "-hide_banner".to_string(),
            "-loglevel".into(),
            "info".into(),
        ];
        args.extend(input_args.iter().cloned());
        args.extend([
            "-c:v".into(),
            encoder.to_string(),
            "-preset".into(),
            "veryfast".into(),
            "-b:v".into(),
            format!("{}k", self.video_bitrate_kbps),
            "-c:a".into(),
            "aac".into(),
        ]);
        if self.format == RecordingFormat::Mp4 {
            // Faststart so partial files remain playable after a crash.
            args.extend(["-movflags".into(), "+faststart".into()]);
        }
        args.extend(["-f".into(), self.format.muxer().into()]);
        args.push(out_path.to_string_lossy().into_owned());
        args
    }
}

/// Stream-copy remux `input.mkv` → `output.mp4` (no re-encode).
pub fn remux_args(input: &Path, output: &Path) -> Vec<String> {
    vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-i".into(),
        input.to_string_lossy().into_owned(),
        "-c".into(),
        "copy".into(),
        "-movflags".into(),
        "+faststart".into(),
        output.to_string_lossy().into_owned(),
    ]
}

/// Metadata of an active recording (for the status payload).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveRecording {
    pub path: String,
    pub format: RecordingFormat,
    pub started_unix_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> RecordingConfig {
        RecordingConfig {
            file_name: "show-01.mkv".into(),
            format: RecordingFormat::Mkv,
            tracks: TrackMode::Mixed,
            encoder: "libx264".into(),
            video_bitrate_kbps: 8000,
        }
    }

    #[test]
    fn validates_extension_and_confines_path() {
        let base = Path::new("/tmp/hub/recordings");
        assert!(cfg().validate(base).is_ok());
        let mut bad_ext = cfg();
        bad_ext.file_name = "show-01.mp4".into();
        assert!(bad_ext.validate(base).is_err());
        let mut escape = cfg();
        escape.file_name = "../evil.mkv".into();
        assert!(escape.validate(base).is_err());
    }

    #[test]
    fn mp4_gets_faststart_and_matroska_muxer_for_mkv() {
        let args = cfg().build_args(Path::new("/tmp/hub/recordings/show-01.mkv"), "libx264", &[]);
        assert!(args.contains(&"matroska".to_string()));
        assert!(!args.iter().any(|a| a == "faststart" || a == "+faststart"));
        let mut mp4 = cfg();
        mp4.format = RecordingFormat::Mp4;
        mp4.file_name = "show-01.mp4".into();
        let args = mp4.build_args(Path::new("/tmp/x.mp4"), "libx264", &[]);
        assert!(args.contains(&"+faststart".to_string()));
    }

    #[test]
    fn remux_is_stream_copy() {
        let args = remux_args(Path::new("/tmp/a.mkv"), Path::new("/tmp/a.mp4"));
        assert!(args.contains(&"copy".to_string()));
        assert!(!args.iter().any(|a| a == "libx264"));
    }
}
