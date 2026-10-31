//! Engine state machine (Phase 1).
//!
//! ```text
//! UNINITIALIZED → INITIALIZING → READY → RUNNING → STOPPING → STOPPED
//!        │              │            │        │          │          │
//!        └──────────────┴────────────┴────────┴──────────┴──────────┴→ ERROR
//! ```
//! `ERROR` is recoverable: a new `initialize_engine` call restarts the cycle.
//! All transitions are validated; illegal ones return [`EngineError::InvalidTransition`].

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// All engine lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EngineState {
    /// Nothing initialised yet (process just started).
    #[default]
    Uninitialized,
    /// Probing FFmpeg / FFprobe / encoders.
    Initializing,
    /// Probes succeeded, ready to run a media test or stream.
    Ready,
    /// A managed media pipeline (Phase 1: test pattern) is running.
    Running,
    /// Graceful shutdown of the pipeline in progress.
    Stopping,
    /// Pipeline stopped cleanly; can be started again.
    Stopped,
    /// A subsystem reported a fatal error; re-initialisation required.
    Error,
}

impl EngineState {
    /// Returns `true` when the engine accepts an `initialize_engine` call.
    pub fn can_initialize(self) -> bool {
        matches!(
            self,
            EngineState::Uninitialized | EngineState::Stopped | EngineState::Error
        )
    }

    /// Returns `true` when a media test / pipeline may start.
    pub fn can_start(self) -> bool {
        matches!(self, EngineState::Ready | EngineState::Stopped)
    }
}

/// Snapshot sent to the frontend over IPC / events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineStatus {
    pub state: EngineState,
    pub ffmpeg_version: Option<String>,
    pub ffprobe_version: Option<String>,
    pub active_encoder: Option<String>,
    pub last_error: Option<String>,
}

impl Default for EngineStatus {
    fn default() -> Self {
        Self {
            state: EngineState::Uninitialized,
            ffmpeg_version: None,
            ffprobe_version: None,
            active_encoder: None,
            last_error: None,
        }
    }
}

/// Shared mutable engine state (Tokio `RwLock` — never held across `.await`
/// of a child process).
#[derive(Debug, Clone, Default)]
pub struct SharedEngine {
    inner: Arc<RwLock<EngineStatus>>,
}

impl SharedEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn status(&self) -> EngineStatus {
        self.inner.read().await.clone()
    }

    /// Validated transition helper. The closure mutates the status while the
    /// write lock is held and must be synchronous.
    pub async fn transition<F>(&self, next: EngineState, apply: F) -> EngineResult<EngineStatus>
    where
        F: FnOnce(&mut EngineStatus),
    {
        let mut guard = self.inner.write().await;
        use EngineState::*;
        let allowed = matches!(
            (guard.state, next),
            (Uninitialized, Initializing)
                | (Stopped, Initializing)
                | (Error, Initializing)
                | (Initializing, Ready)
                | (Initializing, Error)
                | (Ready, Running)
                | (Ready, Error)
                | (Stopped, Running)
                | (Running, Stopping)
                | (Running, Error)
                | (Stopping, Stopped)
                | (Stopping, Error)
        );
        if !allowed {
            return Err(EngineError::InvalidTransition(format!(
                "{:?} -> {:?}",
                guard.state, next
            )));
        }
        guard.state = next;
        if next != EngineState::Error {
            guard.last_error = None;
        }
        apply(&mut guard);
        Ok(guard.clone())
    }

    /// Record a fatal error and move to `ERROR` from any state.
    pub async fn fail(&self, err: &EngineError) -> EngineStatus {
        let mut guard = self.inner.write().await;
        guard.state = EngineState::Error;
        guard.last_error = Some(err.to_string());
        guard.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn full_lifecycle_is_valid() {
        let engine = SharedEngine::new();
        assert_eq!(engine.status().await.state, EngineState::Uninitialized);

        engine
            .transition(EngineState::Initializing, |_| {})
            .await
            .unwrap();
        engine
            .transition(EngineState::Ready, |s| {
                s.ffmpeg_version = Some("7.1".into());
            })
            .await
            .unwrap();
        engine
            .transition(EngineState::Running, |_| {})
            .await
            .unwrap();
        engine
            .transition(EngineState::Stopping, |_| {})
            .await
            .unwrap();
        let stopped = engine
            .transition(EngineState::Stopped, |_| {})
            .await
            .unwrap();
        assert_eq!(stopped.state, EngineState::Stopped);
        // stopped -> running directly is allowed (quick restart), stopped ->
        // ready is not.
        assert!(engine.transition(EngineState::Ready, |_| {}).await.is_err());
    }

    #[tokio::test]
    async fn illegal_jump_is_rejected() {
        let engine = SharedEngine::new();
        let err = engine
            .transition(EngineState::Running, |_| {})
            .await
            .unwrap_err();
        assert!(matches!(err, EngineError::InvalidTransition(_)));
    }

    #[tokio::test]
    async fn error_is_recoverable_via_reinitialize() {
        let engine = SharedEngine::new();
        engine.fail(&EngineError::Timeout(100)).await;
        assert_eq!(engine.status().await.state, EngineState::Error);
        assert!(engine.status().await.state.can_initialize());
        engine
            .transition(EngineState::Initializing, |_| {})
            .await
            .unwrap();
    }
}
