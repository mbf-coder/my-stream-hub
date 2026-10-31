//! `MediaEngine`: owns the Phase 1 lifecycle.
//!
//! Probe FFmpeg/FFprobe/encoders, hold validated config, start/stop the
//! managed test pipeline, surface diagnostics. Never blocks the Tokio runtime.

use crate::errors::{EngineError, EngineResult};
use crate::media::encoders::{self, AvailableEncoder};
use crate::media::ffmpeg::{self, MediaCapabilities};
use crate::media::process_manager::{FfmpegJob, ManagedProcess, ProcessManager};
use crate::state::EngineState;
use crate::state::{EngineStatus, SharedEngine};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Live engine: shared status + discovered capabilities + running child.
pub struct MediaEngine {
    status: SharedEngine,
    capabilities: Arc<Mutex<MediaCapabilities>>,
    encoders: Arc<Mutex<Vec<AvailableEncoder>>>,
    processes: ProcessManager,
    running: Arc<Mutex<Option<ManagedProcess>>>,
}

impl MediaEngine {
    pub fn new(status: SharedEngine) -> Self {
        Self {
            status,
            capabilities: Arc::new(Mutex::new(MediaCapabilities::default())),
            encoders: Arc::new(Mutex::new(Vec::new())),
            processes: ProcessManager::new(),
            running: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn status(&self) -> EngineStatus {
        self.status.status().await
    }

    pub async fn capabilities(&self) -> MediaCapabilities {
        self.capabilities.lock().await.clone()
    }

    pub async fn encoders(&self) -> Vec<AvailableEncoder> {
        self.encoders.lock().await.clone()
    }

    /// INITIALIZING → READY (or ERROR): probe binaries and encoder list.
    pub async fn initialize(&self) -> EngineResult<EngineStatus> {
        if !self.status.status().await.state.can_initialize() {
            return Err(EngineError::InvalidTransition(
                "engine already initialized; stop or reset first".into(),
            ));
        }
        self.status.transition(EngineState::Initializing, |_| {}).await?;
        let ffmpeg_version = match ffmpeg::probe_version(ffmpeg::FFMPEG_BIN).await {
            Ok(v) => Some(v),
            Err(e) => {
                let msg = e.to_string();
                self.status.transition(EngineState::Error, |s| { s.last_error = Some(msg); }).await.ok();
                return Err(e);
            }
        };
        let ffprobe_version = ffmpeg::probe_version(ffmpeg::FFPROBE_BIN).await.ok();
        let encoders_output = ffmpeg::fetch_encoders_output().await.unwrap_or_default();
        let caps = ffmpeg::capabilities_from_encoders(ffmpeg_version.clone(), ffprobe_version.clone(), &encoders_output);
        let found = encoders::detect_available_encoders(&encoders_output);
        let best = encoders::select_best(&found).map(|e| e.name.clone());
        *self.capabilities.lock().await = caps;
        *self.encoders.lock().await = found;
        let status = self.status.transition(EngineState::Ready, |s| {
            s.ffmpeg_version = ffmpeg_version;
            s.ffprobe_version = ffprobe_version;
            s.active_encoder = best;
        }).await?;
        Ok(status)
    }

    /// READY/STOPPED → RUNNING: spawn any validated pipeline job.
    pub async fn start_pipeline(&self, job: &FfmpegJob) -> EngineResult<EngineStatus> {
        if !self.status.status().await.state.can_start() {
            return Err(EngineError::InvalidTransition("initialize the engine before starting a pipeline".into()));
        }
        let child = self.processes.spawn(job, ffmpeg::FFMPEG_BIN).await?;
        *self.running.lock().await = Some(child);
        self.status.transition(EngineState::Running, |_| {}).await
    }

    /// RUNNING → STOPPING → STOPPED with graceful child shutdown.
    pub async fn stop_pipeline(&self) -> EngineResult<EngineStatus> {
        if self.status.status().await.state != EngineState::Running {
            return Err(EngineError::InvalidTransition("no pipeline is running".into()));
        }
        self.status.transition(EngineState::Stopping, |_| {}).await?;
        if let Some(child) = self.running.lock().await.take() {
            child.stop(super::process_manager::DEFAULT_STOP_TIMEOUT).await?;
        }
        self.status.transition(EngineState::Stopped, |_| {}).await
    }

    /// Non-destructive stderr snapshot of the running child (for monitors).
    pub async fn recent_child_stderr(&self) -> Vec<String> {
        self.running.lock().await.as_ref().map(|c| c.recent_stderr()).unwrap_or_default()
    }

    /// READY/STOPPED → RUNNING: spawn the null-muxer media test.
    pub async fn start_media_test(&self) -> EngineResult<EngineStatus> {
        self.start_pipeline(&FfmpegJob::media_test()).await
    }

    /// RUNNING → STOPPING → STOPPED with graceful child shutdown.
    pub async fn stop_media_test(&self) -> EngineResult<EngineStatus> {
        self.stop_pipeline().await
    }
}
