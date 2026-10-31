//! Audio engine — Phase 4.
//!
//! Three-channel mixer (mic / desktop / aux) + master, 3-band EQ, voice FX,
//! routing matrix and a VST3 placeholder that is explicit about being
//! unimplemented (scanning a directory never pretends to load plugins).
//! [`build_audio_filter`] maps the mixer to a real FFmpeg `-af` chain.

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};

pub const MIN_GAIN_DB: f32 = -60.0;
pub const MAX_GAIN_DB: f32 = 12.0;

/// Mixer channels (fixed Phase-4 surface: 3 inputs + master).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChannelId {
    #[default]
    Mic,
    Desktop,
    Aux,
}

impl ChannelId {
    pub fn all() -> [ChannelId; 3] {
        [ChannelId::Mic, ChannelId::Desktop, ChannelId::Aux]
    }
    pub fn label(self) -> &'static str {
        match self {
            ChannelId::Mic => "Microphone",
            ChannelId::Desktop => "Desktop",
            ChannelId::Aux => "Aux",
        }
    }
    /// FFmpeg `amix` input index for the channel.
    pub fn input_index(self) -> usize {
        match self {
            ChannelId::Mic => 0,
            ChannelId::Desktop => 1,
            ChannelId::Aux => 2,
        }
    }
}

fn valid_gain(g: f32) -> EngineResult<()> {
    if !g.is_finite() || !(MIN_GAIN_DB..=MAX_GAIN_DB).contains(&g) {
        return Err(EngineError::InvalidParameter(
            "gain_db".into(),
            format!("gain must be {MIN_GAIN_DB}..={MAX_GAIN_DB} dB"),
        ));
    }
    Ok(())
}

/// One EQ band (low-shelf / peaking / high-shelf by frequency).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EqBand {
    pub freq_hz: u32,
    pub gain_db: f32,
    pub q: f32,
}

impl EqBand {
    pub fn validate(&self) -> EngineResult<()> {
        if !(20..=20000).contains(&self.freq_hz) {
            return Err(EngineError::InvalidParameter(
                "freq_hz".into(),
                "must be 20..=20000".into(),
            ));
        }
        if !(-24.0..=24.0).contains(&self.gain_db) || !self.gain_db.is_finite() {
            return Err(EngineError::InvalidParameter(
                "gain_db".into(),
                "EQ gain must be -24..=+24 dB".into(),
            ));
        }
        if !(0.1..=18.0).contains(&self.q) || !self.q.is_finite() {
            return Err(EngineError::InvalidParameter(
                "q".into(),
                "Q must be 0.1..=18".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Eq3 {
    pub low: EqBand,
    pub mid: EqBand,
    pub high: EqBand,
}

impl Default for Eq3 {
    fn default() -> Self {
        Self {
            low: EqBand {
                freq_hz: 120,
                gain_db: 0.0,
                q: 0.7,
            },
            mid: EqBand {
                freq_hz: 1200,
                gain_db: 0.0,
                q: 1.0,
            },
            high: EqBand {
                freq_hz: 8000,
                gain_db: 0.7,
                q: 0.7,
            },
        }
    }
}
