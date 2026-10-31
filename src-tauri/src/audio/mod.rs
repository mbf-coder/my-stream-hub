//! Audio engine — Phase 4.
//!
//! Three-channel mixer (mic / desktop / aux) + master, 3-band EQ, voice FX,
//! routing matrix and a VST3 placeholder that is explicit about being
//! unimplemented. [`build_audio_filter`] maps the mixer to a real FFmpeg `-af` chain.

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};

pub const MIN_GAIN_DB: f32 = -60.0;
pub const MAX_GAIN_DB: f32 = 12.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChannelId {
    #[default]
    Mic,
    Desktop,
    Aux,
}

impl ChannelId {
    pub fn all() -> [ChannelId; 3] { [ChannelId::Mic, ChannelId::Desktop, ChannelId::Aux] }
    pub fn input_index(self) -> usize {
        match self { ChannelId::Mic => 0, ChannelId::Desktop => 1, ChannelId::Aux => 2 }
    }
}

fn valid_gain(g: f32) -> EngineResult<()> {
    if !g.is_finite() || !(MIN_GAIN_DB..=MAX_GAIN_DB).contains(&g) {
        return Err(EngineError::InvalidParameter("gain_db".into(), format!("gain must be {MIN_GAIN_DB}..={MAX_GAIN_DB} dB")));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EqBand { pub freq_hz: u32, pub gain_db: f32, pub q: f32 }

impl EqBand {
    pub fn validate(&self) -> EngineResult<()> {
        if !(20..=20000).contains(&self.freq_hz) { return Err(EngineError::InvalidParameter("freq_hz".into(), "must be 20..=20000".into())); }
        if !(-24.0..=24.0).contains(&self.gain_db) || !self.gain_db.is_finite() { return Err(EngineError::InvalidParameter("gain_db".into(), "EQ gain must be -24..=+24 dB".into())); }
        if !(0.1..=18.0).contains(&self.q) || !self.q.is_finite() { return Err(EngineError::InvalidParameter("q".into(), "Q must be 0.1..=18".into())); }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Eq3 { pub low: EqBand, pub mid: EqBand, pub high: EqBand }

impl Default for Eq3 {
    fn default() -> Self {
        Self {
            low: EqBand { freq_hz: 120, gain_db: 0.0, q: 0.7 },
            mid: EqBand { freq_hz: 1200, gain_db: 0.0, q: 1.0 },
            high: EqBand { freq_hz: 8000, gain_db: 0.0, q: 0.7 },
        }
    }
}

impl Eq3 {
    pub fn validate(&self) -> EngineResult<()> { self.low.validate()?; self.mid.validate()?; self.high.validate()?; Ok(()) }
    pub fn is_flat(&self) -> bool { [self.low, self.mid, self.high].iter().all(|b| b.gain_db == 0.0) }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum VoiceFx {
    #[default]
    None,
    PitchShift(i16),
    Robot,
    Echo,
}

impl VoiceFx {
    pub fn validate(&self) -> EngineResult<()> {
        if let VoiceFx::PitchShift(c) = self {
            if !(-1200..=1200).contains(c) { return Err(EngineError::InvalidParameter("cents".into(), "pitch shift must be -1200..=+1200 cents".into())); }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Channel { pub id: ChannelId, pub gain_db: f32, pub muted: bool, #[serde(default)] pub peak: f32 }

impl Channel {
    pub fn new(id: ChannelId) -> Self { Self { id, gain_db: 0.0, muted: false, peak: 0.0 } }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mixer {
    pub channels: Vec<Channel>,
    pub master_gain_db: f32,
    pub master_muted: bool,
    pub eq: Eq3,
    pub voice_fx: VoiceFx,
    pub monitor_muted: Vec<ChannelId>,
}

impl Default for Mixer {
    fn default() -> Self {
        Self {
            channels: ChannelId::all().iter().map(|&id| Channel::new(id)).collect(),
            master_gain_db: 0.0, master_muted: false,
            eq: Eq3::default(), voice_fx: VoiceFx::None, monitor_muted: Vec::new(),
        }
    }
}

impl Mixer {
    pub fn channel_mut(&mut self, id: ChannelId) -> EngineResult<&mut Channel> {
        self.channels.iter_mut().find(|c| c.id == id).ok_or_else(|| EngineError::InvalidParameter("channel".into(), "unknown channel".into()))
    }
    pub fn set_gain(&mut self, id: ChannelId, gain_db: f32) -> EngineResult<()> {
        valid_gain(gain_db)?;
        self.channel_mut(id)?.gain_db = gain_db;
        Ok(())
    }
    pub fn set_mute(&mut self, id: ChannelId, muted: bool) -> EngineResult<()> {
        self.channel_mut(id)?.muted = muted;
        Ok(())
    }
    pub fn set_master_gain(&mut self, gain_db: f32) -> EngineResult<()> {
        valid_gain(gain_db)?;
        self.master_gain_db = gain_db;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VstPlugin { pub name: String, pub path: String, pub enabled: bool }

pub fn scan_vst3_dir(dir: &str) -> EngineResult<(Vec<VstPlugin>, String)> {
    if dir.trim().is_empty() { return Err(EngineError::InvalidParameter("vst_dir".into(), "directory must not be empty".into())); }
    Ok((Vec::new(), "VST3 hosting is not implemented in Phase 4; directory accepted for future use.".into()))
}

pub fn build_audio_filter(mixer: &Mixer) -> EngineResult<String> {
    mixer.eq.validate()?;
    mixer.voice_fx.validate()?;
    valid_gain(mixer.master_gain_db)?;
    for ch in &mixer.channels { valid_gain(ch.gain_db)?; }
    let mut parts: Vec<String> = Vec::new();
    for ch in &mixer.channels {
        let gain = if ch.muted { MIN_GAIN_DB } else { ch.gain_db };
        parts.push(format!("[{}:a]volume={gain:.1}dB[ch{}]", ch.id.input_index(), ch.id.input_index()));
    }
    parts.push("[ch0][ch1][ch2]amix=inputs=3:normalize=0[mix]".into());
    if !mixer.eq.is_flat() {
        let curve = |b: &EqBand| format!("frequency={}:width_type=o:width={}:gain={}", b.freq_hz, b.q, b.gain_db);
        parts.push(format!("[mix]anequalizer={}|{}|{}[eq]", curve(&mixer.eq.low), curve(&mixer.eq.mid), curve(&mixer.eq.high)));
    } else {
        parts.push("[mix]anull[eq]".into());
    }
    match mixer.voice_fx {
        VoiceFx::None => parts.push("[eq]anull[fx]".into()),
        VoiceFx::PitchShift(cents) => {
            let ratio = 2f64.powf(f64::from(cents) / 1200.0);
            parts.push(format!("[eq]asetrate=48000*{ratio:.6},aresample=48000[fx]"));
        }
        VoiceFx::Robot => parts.push("[eq]afftdn=nf=-25,atremolo=f=30:d=1.0[fx]".into()),
        VoiceFx::Echo => parts.push("[eq]aecho=0.8:0.7:120|180:0.5|0.35[fx]".into()),
    }
    if mixer.master_muted { parts.push("[fx]volume=-inf[outa]".into()); }
    else { parts.push(format!("[fx]volume={:.1}dB[outa]", mixer.master_gain_db)); }
    Ok(parts.join(";"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gain_bounds_are_enforced() {
        let mut m = Mixer::default();
        assert!(m.set_gain(ChannelId::Mic, 6.0).is_ok());
        assert!(m.set_gain(ChannelId::Mic, 99.0).is_err());
        assert!(m.set_gain(ChannelId::Mic, f32::NAN).is_err());
        assert!(m.set_master_gain(-60.0).is_ok());
    }
    #[test]
    fn flat_mixer_produces_passthrough_chain() {
        let f = build_audio_filter(&Mixer::default()).unwrap();
        assert!(f.contains("amix=inputs=3"));
        assert!(f.contains("[outa]"));
        assert!(!f.contains("anequalizer"));
    }
    #[test]
    fn mute_and_fx_shape_the_chain() {
        let mut m = Mixer::default();
        m.set_mute(ChannelId::Desktop, true).unwrap();
        m.voice_fx = VoiceFx::Echo;
        let f = build_audio_filter(&m).unwrap();
        assert!(f.contains("[1:a]volume=-60.0dB"));
        assert!(f.contains("aecho="));
    }
    #[test]
    fn eq_validation_rejects_absurd_bands() {
        let mut m = Mixer::default();
        m.eq.low.freq_hz = 5;
        assert!(build_audio_filter(&m).is_err());
    }
    #[test]
    fn vst_scan_is_honest() {
        assert!(scan_vst3_dir("").is_err());
        let (plugins, note) = scan_vst3_dir("C:\\Program Files\\Common Files\\VST3").unwrap();
        assert!(plugins.is_empty());
        assert!(note.contains("not implemented"));
    }
}
