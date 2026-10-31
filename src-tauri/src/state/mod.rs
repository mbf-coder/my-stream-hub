//! Engine state machine: UNINITIALIZED → INITIALIZING → READY → RUNNING → STOPPING → STOPPED, plus recoverable ERROR.

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EngineState {
    #[default]
    Uninitialized,
    Initializing,
    Ready,
    Running,
    Stopping,
    Stopped,
    Error,
}

impl EngineState {
    pub fn can_initialize(self) -> bool {
        matches!(self, EngineState::Uninitialized | EngineState::Stopped | EngineState::Error)
    }
    pub fn can_start(self) -> bool {
        matches!(self, EngineState::Ready | EngineState::Stopped)
    }
}

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
        Self { state: EngineState::Uninitialized, ffmpeg_version: None, ffprobe_version: None, active_encoder: None, last_error: None }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SharedEngine { inner: Arc<RwLock<EngineStatus>> }

impl SharedEngine {
    pub fn new() -> Self { Self::default() }
    pub async fn status(&self) -> EngineStatus { self.inner.read().await.clone() }
    pub async fn transition<F>(&self, next: EngineState, apply: F) -> EngineResult<EngineStatus>
    where F: FnOnce(&mut EngineStatus) {
        let mut guard = self.inner.write().await;
        use EngineState::*;
        let allowed = matches!(
            (guard.state, next),
            (Uninitialized, Initializing) | (Stopped, Initializing) | (Error, Initializing)
                | (Initializing, Ready) | (Initializing, Error)
                | (Ready, Running) | (Ready, Error) | (Stopped, Running)
                | (Running, Stopping) | (Running, Error)
                | (Stopping, Stopped) | (Stopping, Error)
        );
        if !allowed {
            return Err(EngineError::InvalidTransition(format!("{:?} -> {:?}", guard.state, next)));
        }
        guard.state = next;
        if next != EngineState::Error { guard.last_error = None; }
        apply(&mut guard);
        Ok(guard.clone())
    }
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
        engine.transition(EngineState::Initializing, |_| {}).await.unwrap();
        engine.transition(EngineState::Ready, |s| { s.ffmpeg_version = Some("7.1".into()); }).await.unwrap();
        engine.transition(EngineState::Running, |_| {}).await.unwrap();
        engine.transition(EngineState::Stopping, |_| {}).await.unwrap();
        let stopped = engine.transition(EngineState::Stopped, |_| {}).await.unwrap();
        assert_eq!(stopped.state, EngineState::Stopped);
        assert!(engine.transition(EngineState::Ready, |_| {}).await.is_err());
    }
    #[tokio::test]
    async fn illegal_jump_is_rejected() {
        let engine = SharedEngine::new();
        let err = engine.transition(EngineState::Running, |_| {}).await.unwrap_err();
        assert!(matches!(err, EngineError::InvalidTransition(_)));
    }
    #[tokio::test]
    async fn error_is_recoverable_via_reinitialize() {
        let engine = SharedEngine::new();
        engine.fail(&EngineError::Timeout(100)).await;
        assert_eq!(engine.status().await.state, EngineState::Error);
        assert!(engine.status().await.state.can_initialize());
        engine.transition(EngineState::Initializing, |_| {}).await.unwrap();
    }
}
