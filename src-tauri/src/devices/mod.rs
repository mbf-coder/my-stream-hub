//! Device enumeration — DirectShow parsing (pure, tested); live probe on Windows only.

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaptureDevice { pub name: String, pub kind: DeviceKind, pub detail: String }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind { Audio, Video }

pub fn parse_dshow_devices(output: &str) -> (Vec<String>, Vec<String>) {
    let mut video = Vec::new();
    let mut audio = Vec::new();
    let mut section: Option<&str> = None;
    for line in output.lines() {
        let l = line.trim();
        if l.contains("DirectShow video devices") { section = Some("video"); continue; }
        if l.contains("DirectShow audio devices") { section = Some("audio"); continue; }
        if let (Some(open), Some(close)) = (l.find('"'), l.rfind('"')) {
            if open < close {
                let name = l[open + 1..close].to_string();
                if !name.is_empty() {
                    match section { Some("video") => video.push(name), Some("audio") => audio.push(name), _ => {} }
                }
            }
        }
    }
    (video, audio)
}

pub async fn list_devices(kind: DeviceKind) -> EngineResult<Vec<CaptureDevice>> {
    if cfg!(not(target_os = "windows")) {
        return Err(EngineError::DeviceEnumeration("device probing requires Windows 11 (dshow)".into()));
    }
    let out = tokio::time::timeout(Duration::from_millis(10_000),
        Command::new("ffmpeg").args(["-hide_banner", "-list_devices", "true", "-f", "dshow", "-i", "dummy"]).output())
        .await.map_err(|_| EngineError::Timeout(10_000))?
        .map_err(|e| EngineError::DeviceEnumeration(e.to_string()))?;
    let text = String::from_utf8_lossy(&out.stderr).into_owned();
    let (video, audio) = parse_dshow_devices(&text);
    let names = match kind { DeviceKind::Video => video, DeviceKind::Audio => audio };
    Ok(names.into_iter().map(|name| CaptureDevice { name: name.clone(), kind, detail: format!("dshow: {name}") }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE: &str = "ffmpeg version 7.1\\n[dshow] DirectShow video devices\\n[dshow]  \\"Integrated Camera\\"\\n[dshow] DirectShow audio devices\\n[dshow]  \\"Microphone\\"\\n";
    #[test]
    fn splits_video_and_audio_sections() {
        let (video, audio) = parse_dshow_devices(FIXTURE);
        assert_eq!(video, vec!["Integrated Camera"]);
        assert_eq!(audio, vec!["Microphone"]);
    }
    #[test]
    fn empty_output_yields_empty_lists() {
        assert_eq!(parse_dshow_devices(""), (vec![], vec![]));
    }
}
