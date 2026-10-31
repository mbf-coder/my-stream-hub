//! Structured engine errors (Phase 1).
//!
//! Every failure crossing the IPC boundary maps to one of these variants so
//! the frontend can show actionable messages without leaking internals
//! (paths, keys, tokens are never included in `Display` output).

use thiserror::Error;

/// Result alias used across all engine subsystems.
pub type EngineResult<T> = Result<T, EngineError>;

/// All recoverable engine failures.
#[derive(Debug, Error, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum EngineError {
    /// Illegal state-machine transition (e.g. start while already running).
    #[error("invalid state transition: {0}")]
    InvalidTransition(String),

    /// FFmpeg binary missing or unusable.
    #[error("ffmpeg unavailable: {0}")]
    FfmpegUnavailable(String),

    /// FFprobe binary missing or unusable.
    #[error("ffprobe unavailable: {0}")]
    FfprobeUnavailable(String),

    /// Requested encoder is not available on this machine.
    #[error("encoder unavailable: {0}")]
    EncoderUnavailable(String),

    /// A managed child process failed.
    #[error("process failure: {0}")]
    Process(String),

    /// Operation exceeded its deadline.
    #[error("operation timed out after {0}ms")]
    Timeout(u64),

    /// IPC / configuration parameter rejected by validation.
    #[error("invalid parameter '{0}': {1}")]
    InvalidParameter(String, String),

    /// Device enumeration failed (non-fatal: engine keeps running).
    #[error("device enumeration failed: {0}")]
    DeviceEnumeration(String),

    /// Secure credential storage failed.
    #[error("credential store failure")]
    CredentialStore,

    /// Catch-all for unexpected failures (message only, no secrets).
    #[error("internal error: {0}")]
    Internal(String),
}
