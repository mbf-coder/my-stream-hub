//! Encoder discovery with automatic best-pick (Phase 1).
//!
//! Priority: NVENC → AMF → QSV → libx264 (software fallback).

use serde::{Deserialize, Serialize};

/// Encoder families the engine knows about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncoderKind {
    NvencH264,
    AmfH264,
    QsvH264,
    Libx264,
}

impl EncoderKind {
    /// Token as it appears in `ffmpeg -encoders`.
    pub fn token(self) -> &'static str {
        match self {
            EncoderKind::NvencH264 => "h264_nvenc",
            EncoderKind::AmfH264 => "h264_amf",
            EncoderKind::QsvH264 => "h264_qsv",
            EncoderKind::Libx264 => "libx264",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EncoderKind::NvencH264 => "NVIDIA NVENC H.264",
            EncoderKind::AmfH264 => "AMD AMF H.264",
            EncoderKind::QsvH264 => "Intel Quick Sync H.264",
            EncoderKind::Libx264 => "Software x264",
        }
    }

    pub fn is_hardware(self) -> bool {
        !matches!(self, EncoderKind::Libx264)
    }

    /// Preference order, best first.
    pub fn preference_order() -> &'static [EncoderKind] {
        &[
            EncoderKind::NvencH264,
            EncoderKind::AmfH264,
            EncoderKind::QsvH264,
            EncoderKind::Libx264,
        ]
    }
}

/// One verified-available encoder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvailableEncoder {
    pub kind: EncoderKind,
    pub name: String,
    pub hardware: bool,
}

/// Scan `-encoders` output for the known tokens (case-insensitive).
pub fn detect_available_encoders(encoders_output: &str) -> Vec<AvailableEncoder> {
    let lower = encoders_output.to_lowercase();
    EncoderKind::preference_order()
        .iter()
        .filter(|k| lower.contains(k.token()))
        .map(|k| AvailableEncoder {
            kind: *k,
            name: k.token().to_string(),
            hardware: k.is_hardware(),
        })
        .collect()
}

/// Pick the best available encoder, if any.
pub fn select_best(encoders: &[AvailableEncoder]) -> Option<AvailableEncoder> {
    encoders.first().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = " V..... h264_nvenc  NVIDIA NVENC H.264\n V..... libx264  libx264 H.264\n";

    #[test]
    fn detects_only_listed_encoders() {
        let found = detect_available_encoders(SAMPLE);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].kind, EncoderKind::NvencH264);
        assert_eq!(found[1].kind, EncoderKind::Libx264);
        assert!(found[0].hardware);
        assert!(!found[1].hardware);
    }

    #[test]
    fn prefers_hardware_over_software() {
        let found = detect_available_encoders(" V..... h264_qsv Intel\n V..... libx264 x264\n");
        let best = select_best(&found).unwrap();
        assert_eq!(best.kind, EncoderKind::QsvH264);
    }

    #[test]
    fn software_fallback_when_no_hardware() {
        let found = detect_available_encoders(" V..... libx264 x264\n");
        assert_eq!(select_best(&found).unwrap().kind, EncoderKind::Libx264);
    }

    #[test]
    fn empty_when_nothing_matches() {
        let found = detect_available_encoders("no encoders here\n");
        assert!(found.is_empty());
        assert!(select_best(&found).is_none());
    }
}
