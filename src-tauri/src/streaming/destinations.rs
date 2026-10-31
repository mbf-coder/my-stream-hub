//! Multi-destination streaming — Phase 5.

use crate::errors::{EngineError, EngineResult};
use crate::monitoring;
#[cfg(feature = "tauri-app")]
use crate::security::KEYRING_SERVICE;
use crate::security::{self};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    #[default]
    Twitch,
    YouTube,
    Kick,
    Custom,
}

impl Platform {
    pub fn ingest_preset(self) -> Option<&'static str> {
        match self {
            Platform::Twitch => Some("rtmp://live.twitch.tv/app"),
            Platform::YouTube => Some("rtmps://a.rtmp.youtube.com/live2"),
            Platform::Kick => Some("rtmp://fa723fc1b171.global-contribute.live-video.net/app"),
            Platform::Custom => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Destination {
    pub id: String,
    pub platform: Platform,
    pub label: String,
    pub url: String,
    pub enabled: bool,
}

impl Destination {
    pub fn validate_new(label: &str, url: &str) -> EngineResult<()> {
        if label.trim().is_empty() || label.len() > 64 {
            return Err(EngineError::InvalidParameter("label".into(), "label must be 1..=64 chars".into()));
        }
        security::validate_rtmp_url(url).map_err(|m| EngineError::InvalidParameter("url".into(), m))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OAuthToken {
    pub platform: Platform,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
}

#[cfg(feature = "tauri-app")]
fn oauth_account(platform: Platform) -> String {
    format!("oauth-{}", serde_json::to_string(&platform).unwrap_or_default().replace('"', ""))
}

#[cfg(feature = "tauri-app")]
fn keyring_entry(account: &str) -> EngineResult<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, account).map_err(|_| EngineError::CredentialStore)
}

pub fn store_oauth_token(token: &OAuthToken) -> EngineResult<()> {
    if token.access_token.trim().is_empty() || token.access_token.len() > 8192 {
        return Err(EngineError::InvalidParameter("access_token".into(), "token must be 1..=8192 chars".into()));
    }
    #[cfg(feature = "tauri-app")]
    {
        let json = serde_json::to_string(token).map_err(|_| EngineError::CredentialStore)?;
        keyring_entry(&oauth_account(token.platform))?.set_password(&json).map_err(|_| EngineError::CredentialStore)?;
        Ok(())
    }
    #[cfg(not(feature = "tauri-app"))]
    {
        let _ = token;
        Err(EngineError::CredentialStore)
    }
}

pub fn has_valid_oauth_token(platform: Platform, now_unix_secs: u64) -> bool {
    #[cfg(feature = "tauri-app")]
    {
        let Ok(entry) = keyring_entry(&oauth_account(platform)) else { return false; };
        let Ok(json) = entry.get_password() else { return false; };
        let Ok(token): Result<OAuthToken, _> = serde_json::from_str(&json) else { return false; };
        return !token.access_token.is_empty() && token.expires_at > now_unix_secs;
    }
    #[cfg(not(feature = "tauri-app"))]
    {
        let _ = (platform, now_unix_secs);
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StreamMonitor {
    pub bitrate_kbps: f32,
    pub fps: f32,
    pub dropped_frames: u64,
    pub uptime_secs: u64,
    pub connected: bool,
}

impl StreamMonitor {
    pub fn ingest_line(&mut self, line: &str) {
        if let Some(p) = monitoring::parse_progress_line(line) {
            self.bitrate_kbps = p.bitrate_kbps;
            self.fps = p.fps;
            self.uptime_secs = p.elapsed_secs;
            self.connected = true;
        }
        if line.contains("dup=") || line.contains("drop=") { self.dropped_frames += 1; }
    }
}

pub fn assemble_ingest(base_url: &str, stream_key: &str) -> EngineResult<String> {
    if stream_key.trim().is_empty() {
        return Err(EngineError::InvalidParameter("stream_key".into(), "no stream key configured".into()));
    }
    let full = format!("{}/{}", base_url.trim_end_matches('/'), stream_key.trim());
    security::validate_rtmp_url(&full).map_err(|m| EngineError::InvalidParameter("ingest".into(), m))?;
    Ok(full)
}

#[derive(Debug, Default)]
pub struct DestinationStore {
    items: Vec<Destination>,
    next: u32,
}

impl DestinationStore {
    pub fn list(&self) -> Vec<Destination> { self.items.clone() }
    pub fn find(&self, id: &str) -> EngineResult<Destination> {
        self.items.iter().find(|d| d.id == id).cloned().ok_or_else(|| EngineError::InvalidParameter("destination_id".into(), "destination not found".into()))
    }
    pub fn add(&mut self, platform: Platform, label: String, url: String) -> EngineResult<Destination> {
        Destination::validate_new(&label, &url)?;
        self.next += 1;
        let dest = Destination { id: format!("dest-{}", self.next), platform, label: label.trim().to_string(), url, enabled: true };
        self.items.push(dest.clone());
        Ok(dest)
    }
    pub fn remove(&mut self, id: &str) -> EngineResult<()> {
        let pos = self.items.iter().position(|d| d.id == id).ok_or_else(|| EngineError::InvalidParameter("destination_id".into(), "destination not found".into()))?;
        self.items.remove(pos);
        Ok(())
    }
    pub fn toggle(&mut self, id: &str) -> EngineResult<bool> {
        let dest = self.items.iter_mut().find(|d| d.id == id).ok_or_else(|| EngineError::InvalidParameter("destination_id".into(), "destination not found".into()))?;
        dest.enabled = !dest.enabled;
        Ok(dest.enabled)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct StreamRuntime {
    pub active_destination_id: Option<String>,
    pub started_unix_secs: Option<u64>,
    pub monitor: StreamMonitor,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_are_rtmp() {
        assert!(Platform::Twitch.ingest_preset().unwrap().starts_with("rtmp"));
        assert!(Platform::YouTube.ingest_preset().unwrap().starts_with("rtmps"));
        assert!(Platform::Custom.ingest_preset().is_none());
    }
    #[test]
    fn destination_validation() {
        assert!(Destination::validate_new("Main", "rtmp://live.twitch.tv/app").is_ok());
        assert!(Destination::validate_new("", "rtmp://live.twitch.tv/app").is_err());
        assert!(Destination::validate_new("Main", "https://x.com/live").is_err());
    }
    #[test]
    fn ingest_assembly_needs_key() {
        assert!(assemble_ingest("rtmp://live.twitch.tv/app", "").is_err());
        let full = assemble_ingest("rtmp://live.twitch.tv/app/", "live_abc").unwrap();
        assert_eq!(full, "rtmp://live.twitch.tv/app/live_abc");
    }
    #[test]
    fn monitor_feeds_from_progress() {
        let mut m = StreamMonitor::default();
        assert!(!m.connected);
        m.ingest_line("frame= 1234 fps= 60 q=28.0 size=    5120kB time=00:00:20.00 bitrate=2048.0kbits/s speed=1.0x");
        assert!(m.connected);
        assert_eq!(m.uptime_secs, 20);
    }
}
