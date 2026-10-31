//! Event payloads emitted towards the frontend (Phase 1).
//!
//! Event names are stable IPC contracts:
//! `engine-state`, `engine-error`, `engine-stats`, `engine-progress`.

use serde::{Deserialize, Serialize};

/// Channel names — single source of truth shared with the TS `services/tauri.ts`.
pub mod names {
    pub const ENGINE_STATE: &str = "engine-state";
    pub const ENGINE_ERROR: &str = "engine-error";
    pub const ENGINE_STATS: &str = "engine-stats";
    pub const ENGINE_PROGRESS: &str = "engine-progress";
    pub const SCENE_CHANGED: &str = "scene-changed";
    pub const MIXER_LEVELS: &str = "mixer-levels";
    pub const STREAM_STATUS: &str = "stream-status";
    pub const RECORD_STATUS: &str = "record-status";
}

/// Scene bus snapshot broadcast after every rundown mutation (Phase 2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneChanged {
    pub preview_id: Option<String>,
    pub program_id: Option<String>,
    pub scene_count: usize,
}

/// Periodic mixer peaks for the UI meters, 0.0..=1.0 (Phase 4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MixerLevels {
    pub mic: f32,
    pub desktop: f32,
    pub aux: f32,
    pub master: f32,
}

/// Stream health broadcast (Phase 5).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamStatusPayload {
    pub live: bool,
    pub destination_label: Option<String>,
    pub bitrate_kbps: f32,
    pub dropped_frames: u64,
    pub uptime_secs: u64,
}

/// Recording state broadcast (Phase 6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordStatusPayload {
    pub recording: bool,
    pub path: Option<String>,
}

/// Periodic pipeline statistics (Phase 1: parsed from FFmpeg stderr progress).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineStats {
    pub frame: u64,
    pub fps: f32,
    pub bitrate_kbps: f32,
    pub elapsed_secs: u64,
}

/// Progress of a bounded operation (initialisation, media test shutdown).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineProgress {
    pub operation: String,
    pub completed: u8,
    pub total: u8,
}
