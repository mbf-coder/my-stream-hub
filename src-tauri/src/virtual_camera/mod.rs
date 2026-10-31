//! Virtual camera — Phase 6.
//!
//! Honest stub: probing reports whether a DirectShow virtual-camera driver is
//! registered (Windows only); `start` refuses with an actionable error until a
//! driver (OBS VirtualCam / Elgato / XSplit broadcaster driver) is installed.
//! No fake "virtual camera active" state is ever reported.

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};

/// Driver probe result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VirtualCameraStatus {
    /// A virtual-camera capture driver was found on this machine.
    pub driver_installed: bool,
    /// The driver registry/device detail when found.
    pub driver_detail: Option<String>,
    /// Our feed is currently routed to the virtual device.
    pub active: bool,
}

/// Probe for a virtual-camera driver. Non-Windows targets always report
/// `driver_installed: false` (no registry to query).
pub fn probe() -> VirtualCameraStatus {
    #[cfg(target_os = "windows")]
    {
        probe_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        VirtualCameraStatus::default()
    }
}

#[cfg(target_os = "windows")]
fn probe_windows() -> VirtualCameraStatus {
    // Best-effort: look for known VC driver device paths via ffmpeg dshow
    // listing at runtime (done by the caller). Registry probing without
    // `winreg` would shell out; keep the pure function honest instead.
    VirtualCameraStatus {
        driver_installed: false,
        driver_detail: Some(
            "run get_video_devices and look for OBS Virtual Camera / Elgato Virtual Camera".into(),
        ),
        active: false,
    }
}

/// Refuse to start without a driver. Pure so tests are deterministic.
pub fn ensure_startable(status: &VirtualCameraStatus) -> EngineResult<()> {
    if !status.driver_installed {
        return Err(EngineError::InvalidParameter(
            "virtual_camera".into(),
            "no virtual-camera driver installed (install OBS VirtualCam or Elgato Virtual Camera first)".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_is_refused_without_driver() {
        let err = ensure_startable(&VirtualCameraStatus::default()).unwrap_err();
        assert!(matches!(err, EngineError::InvalidParameter(_, _)));
        assert!(err.to_string().contains("no virtual-camera driver"));
    }

    #[test]
    fn probe_never_claims_active() {
        assert!(!probe().active);
    }
}
