//! Typed IPC surface — Phases 1–7 (49 commands).

use crate::audio::{self, ChannelId, Eq3, Mixer, VoiceFx, VstPlugin};
use crate::compositor::{self, CanvasPreset, Layer};
use crate::configuration::{config_path_for_dir, AppConfig};
use crate::devices::{self, CaptureDevice, DeviceKind};
use crate::errors::{EngineError, EngineResult};
use crate::events::{RecordStatusPayload, StreamStatusPayload};
use crate::media::encoders::AvailableEncoder;
use crate::media::engine::MediaEngine;
use crate::media::ffmpeg::MediaCapabilities;
use crate::media::process_manager::FfmpegJob;
use crate::recording::{ActiveRecording, RecordingConfig};
use crate::scenes::{Scene, SceneManager, Source, SourceKind, Transition};
use crate::security;
use crate::state::{EngineState, EngineStatus, SharedEngine};
use crate::streaming::destinations::{self, Destination, DestinationStore, OAuthToken, Platform, StreamRuntime};
use crate::streaming::StreamConfig;
use crate::virtual_camera::{self, VirtualCameraStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub struct AppState {
    pub engine: Mutex<MediaEngine>,
    pub shared: SharedEngine,
    pub scenes: SceneManager,
    pub mixer: RwLock<Mixer>,
    pub destinations: RwLock<DestinationStore>,
    pub stream: RwLock<StreamRuntime>,
    pub recording: RwLock<Option<ActiveRecording>>,
    pub settings: RwLock<AppConfig>,
}

impl AppState {
    pub fn new() -> Self {
        let shared = SharedEngine::new();
        Self {
            engine: Mutex::new(MediaEngine::new(shared.clone())),
            shared,
            scenes: SceneManager::new(),
            mixer: RwLock::new(Mixer::default()),
            destinations: RwLock::new(DestinationStore::default()),
            stream: RwLock::new(StreamRuntime::default()),
            recording: RwLock::new(None),
            settings: RwLock::new(AppConfig::default()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self { Self::new() }
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_engine_status(state: tauri_state<'_, AppState>) -> EngineResult<EngineStatus> {
    Ok(state.shared.status().await)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn initialize_engine(state: tauri_state<'_, AppState>) -> EngineResult<EngineStatus> {
    state.engine.lock().await.initialize().await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_media_capabilities(state: tauri_state<'_, AppState>) -> EngineResult<MediaCapabilities> {
    Ok(state.engine.lock().await.capabilities().await)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_available_encoders(state: tauri_state<'_, AppState>) -> EngineResult<Vec<AvailableEncoder>> {
    Ok(state.engine.lock().await.encoders().await)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_audio_devices(state: tauri_state<'_, AppState>) -> EngineResult<Vec<CaptureDevice>> {
    let _ = &state;
    devices::list_devices(DeviceKind::Audio).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_video_devices(state: tauri_state<'_, AppState>) -> EngineResult<Vec<CaptureDevice>> {
    let _ = &state;
    devices::list_devices(DeviceKind::Video).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn start_media_test(state: tauri_state<'_, AppState>) -> EngineResult<EngineStatus> {
    state.engine.lock().await.start_media_test().await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn stop_media_test(state: tauri_state<'_, AppState>) -> EngineResult<EngineStatus> {
    state.engine.lock().await.stop_media_test().await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn list_scenes(state: tauri_state<'_, AppState>) -> EngineResult<Vec<Scene>> {
    Ok(state.scenes.list_scenes().await)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn create_scene(state: tauri_state<'_, AppState>, name: String) -> EngineResult<Scene> {
    state.scenes.create_scene(name).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn delete_scene(state: tauri_state<'_, AppState>, scene_id: String) -> EngineResult<()> {
    state.scenes.delete_scene(&scene_id).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn rename_scene(state: tauri_state<'_, AppState>, scene_id: String, name: String) -> EngineResult<Scene> {
    state.scenes.rename_scene(&scene_id, name).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn add_source(state: tauri_state<'_, AppState>, scene_id: String, name: String, kind: SourceKind) -> EngineResult<Source> {
    state.scenes.add_source(&scene_id, name, kind).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn remove_source(state: tauri_state<'_, AppState>, scene_id: String, source_id: String) -> EngineResult<()> {
    state.scenes.remove_source(&scene_id, &source_id).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn toggle_source(state: tauri_state<'_, AppState>, scene_id: String, source_id: String) -> EngineResult<bool> {
    state.scenes.toggle_source(&scene_id, &source_id).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn move_source(state: tauri_state<'_, AppState>, scene_id: String, source_id: String, to_index: usize) -> EngineResult<Vec<Source>> {
    state.scenes.move_source(&scene_id, &source_id, to_index).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_preview_scene(state: tauri_state<'_, AppState>, scene_id: String) -> EngineResult<()> {
    state.scenes.set_preview(&scene_id).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_program_scene(state: tauri_state<'_, AppState>, scene_id: String) -> EngineResult<()> {
    state.scenes.set_program(&scene_id).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn take_transition(state: tauri_state<'_, AppState>) -> EngineResult<String> {
    state.scenes.take_transition().await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_transition(state: tauri_state<'_, AppState>, transition: Transition) -> EngineResult<Transition> {
    state.scenes.set_transition(transition).await
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn build_compositor_filter(state: tauri_state<'_, AppState>, canvas: CanvasPreset, layers: Vec<Layer>) -> EngineResult<String> {
    let _ = &state;
    compositor::build_filter_complex(canvas, &layers)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_mixer(state: tauri_state<'_, AppState>) -> EngineResult<Mixer> {
    Ok(state.mixer.read().await.clone())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_channel_gain(state: tauri_state<'_, AppState>, channel: ChannelId, gain_db: f32) -> EngineResult<Mixer> {
    let mut mixer = state.mixer.write().await;
    mixer.set_gain(channel, gain_db)?;
    Ok(mixer.clone())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_channel_mute(state: tauri_state<'_, AppState>, channel: ChannelId, muted: bool) -> EngineResult<Mixer> {
    let mut mixer = state.mixer.write().await;
    mixer.set_mute(channel, muted)?;
    Ok(mixer.clone())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_master_gain(state: tauri_state<'_, AppState>, gain_db: f32) -> EngineResult<Mixer> {
    let mut mixer = state.mixer.write().await;
    mixer.set_master_gain(gain_db)?;
    Ok(mixer.clone())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_master_mute(state: tauri_state<'_, AppState>, muted: bool) -> EngineResult<Mixer> {
    let mut mixer = state.mixer.write().await;
    mixer.master_muted = muted;
    Ok(mixer.clone())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_eq(state: tauri_state<'_, AppState>, eq: Eq3) -> EngineResult<Mixer> {
    eq.validate()?;
    let mut mixer = state.mixer.write().await;
    mixer.eq = eq;
    Ok(mixer.clone())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_voice_fx(state: tauri_state<'_, AppState>, fx: VoiceFx) -> EngineResult<Mixer> {
    fx.validate()?;
    let mut mixer = state.mixer.write().await;
    mixer.voice_fx = fx;
    Ok(mixer.clone())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn scan_vst3(state: tauri_state<'_, AppState>, dir: String) -> EngineResult<(Vec<VstPlugin>, String)> {
    let _ = &state;
    audio::scan_vst3_dir(&dir)
}
#[derive(Debug, Clone, serde::Deserialize)]
pub enum CaptureInput {
    TestPattern,
    Device { video: String, audio: Option<String> },
}
fn validate_device_name(label: &str, name: &str) -> EngineResult<()> {
    if name.trim().is_empty() || name.len() > 256 || name.contains('"') {
        return Err(EngineError::InvalidParameter(label.into(), "device name must be 1..=256 chars with no quotes".into()));
    }
    Ok(())
}
fn capture_input_args(input: &CaptureInput) -> EngineResult<Vec<String>> {
    match input {
        CaptureInput::TestPattern => Ok(vec!["-f".into(), "lavfi".into(), "-i".into(), "testsrc=size=1920x1080:rate=30".into(), "-f".into(), "lavfi".into(), "-i".into(), "sine=frequency=1000:sample_rate=48000".into()]),
        CaptureInput::Device { video, audio } => {
            validate_device_name("video", video)?;
            let spec = match audio {
                Some(a) => { validate_device_name("audio", a)?; format!("video=\"{video}\":audio=\"{a}\"") }
                None => format!("video=\"{video}\""),
            };
            Ok(vec!["-f".into(), "dshow".into(), "-i".into(), spec])
        }
    }
}
fn resolve_encoder(settings_encoder: &str, active: Option<&str>) -> String {
    if settings_encoder != "auto" { return settings_encoder.to_string(); }
    active.unwrap_or("libx264").to_string()
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn list_destinations(state: tauri_state<'_, AppState>) -> EngineResult<Vec<Destination>> {
    Ok(state.destinations.read().await.list())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn add_destination(state: tauri_state<'_, AppState>, platform: Platform, label: String, url: String) -> EngineResult<Destination> {
    state.destinations.write().await.add(platform, label, url)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn remove_destination(state: tauri_state<'_, AppState>, destination_id: String) -> EngineResult<()> {
    state.destinations.write().await.remove(&destination_id)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn toggle_destination(state: tauri_state<'_, AppState>, destination_id: String) -> EngineResult<bool> {
    state.destinations.write().await.toggle(&destination_id)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn set_stream_key(state: tauri_state<'_, AppState>, key: String) -> EngineResult<bool> {
    let _ = &state;
    security::store_stream_key(&key)?;
    Ok(true)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn store_oauth_token(state: tauri_state<'_, AppState>, token: OAuthToken) -> EngineResult<bool> {
    let _ = &state;
    destinations::store_oauth_token(&token)?;
    Ok(true)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn oauth_status(state: tauri_state<'_, AppState>, platform: Platform) -> EngineResult<bool> {
    let _ = &state;
    Ok(destinations::has_valid_oauth_token(platform, now_unix_secs()))
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn start_stream(state: tauri_state<'_, AppState>, destination_id: String, input: CaptureInput) -> EngineResult<EngineStatus> {
    let dest = state.destinations.read().await.find(&destination_id)?;
    if !dest.enabled {
        return Err(EngineError::InvalidParameter("destination_id".into(), "destination is disabled".into()));
    }
    let key = security::load_stream_key().map_err(|_| EngineError::InvalidParameter("stream_key".into(), "no stream key in credential store — save it first".into()))?;
    let ingest = destinations::assemble_ingest(&dest.url, &key)?;
    let status_snapshot = state.shared.status().await;
    let settings = state.settings.read().await;
    let cfg = StreamConfig {
        url: dest.url.clone(),
        encoder: resolve_encoder(&settings.stream.encoder, status_snapshot.active_encoder.as_deref()),
        width: settings.output_width, height: settings.output_height, fps: settings.fps,
        video_bitrate_kbps: settings.stream.video_bitrate_kbps,
        audio_bitrate_kbps: settings.stream.audio_bitrate_kbps,
    };
    let stream_args = cfg.output_args(&ingest)?;
    let mut args = capture_input_args(&input)?;
    if cfg.encoder == "libx264" { args.extend(["-preset".to_string(), "veryfast".to_string()]); }
    args.extend(stream_args);
    let live = state.engine.lock().await.start_pipeline(&FfmpegJob::new(args)).await?;
    let mut rt = state.stream.write().await;
    rt.active_destination_id = Some(dest.id.clone());
    rt.started_unix_secs = Some(now_unix_secs());
    rt.monitor = destinations::StreamMonitor::default();
    Ok(live)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn stop_stream(state: tauri_state<'_, AppState>) -> EngineResult<EngineStatus> {
    if state.stream.read().await.active_destination_id.is_none() {
        return Err(EngineError::InvalidTransition("no stream is live".into()));
    }
    let running = state.engine.lock().await.status().await.state == EngineState::Running;
    let status = if running { state.engine.lock().await.stop_pipeline().await? } else { state.engine.lock().await.status().await };
    *state.stream.write().await = StreamRuntime::default();
    Ok(status)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_stream_status(state: tauri_state<'_, AppState>) -> EngineResult<StreamStatusPayload> {
    let running = state.engine.lock().await.status().await.state == EngineState::Running;
    let stderr = state.engine.lock().await.recent_child_stderr().await;
    let mut rt = state.stream.write().await;
    if running { for line in stderr.iter().rev().take(30) { rt.monitor.ingest_line(line); } }
    else { rt.monitor.connected = false; }
    let label = match &rt.active_destination_id {
        Some(id) => state.destinations.read().await.find(id).ok().map(|d| d.label),
        None => None,
    };
    let uptime = rt.started_unix_secs.map(|t| now_unix_secs().saturating_sub(t)).unwrap_or(0);
    Ok(StreamStatusPayload { live: running && rt.active_destination_id.is_some(), destination_label: label, bitrate_kbps: rt.monitor.bitrate_kbps, dropped_frames: rt.monitor.dropped_frames, uptime_secs: uptime })
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn start_recording(state: tauri_state<'_, AppState>, config: RecordingConfig, input: CaptureInput, base_dir: String) -> EngineResult<EngineStatus> {
    if state.recording.read().await.is_some() {
        return Err(EngineError::InvalidTransition("recording already in progress".into()));
    }
    let out = config.validate(&PathBuf::from(&base_dir))?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| EngineError::Process(format!("create recordings dir: {e}")))?;
    }
    let status_snapshot = state.shared.status().await;
    let rec_encoder = { let settings = state.settings.read().await; resolve_encoder(&settings.stream.encoder, status_snapshot.active_encoder.as_deref()) };
    let mut args = capture_input_args(&input)?;
    args.extend(config.build_args(&out, &rec_encoder, &[]));
    let running = state.engine.lock().await.start_pipeline(&FfmpegJob::new(args)).await?;
    *state.recording.write().await = Some(ActiveRecording { path: out.to_string_lossy().into_owned(), format: config.format, started_unix_secs: now_unix_secs() });
    Ok(running)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn stop_recording(state: tauri_state<'_, AppState>) -> EngineResult<EngineStatus> {
    if state.recording.read().await.is_none() {
        return Err(EngineError::InvalidTransition("no recording in progress".into()));
    }
    let running = state.engine.lock().await.status().await.state == EngineState::Running;
    let status = if running { state.engine.lock().await.stop_pipeline().await? } else { state.engine.lock().await.status().await };
    *state.recording.write().await = None;
    Ok(status)
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_recording_status(state: tauri_state<'_, AppState>) -> EngineResult<RecordStatusPayload> {
    let rec = state.recording.read().await.clone();
    let running = state.engine.lock().await.status().await.state == EngineState::Running;
    Ok(RecordStatusPayload { recording: rec.is_some() && running, path: rec.map(|r| r.path) })
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn remux_recording(state: tauri_state<'_, AppState>, input: String, output: String, base_dir: String) -> EngineResult<String> {
    let _ = &state;
    let base = PathBuf::from(&base_dir);
    let src = crate::security::validate_path_inside_base(&base, &input)?;
    let dst = crate::security::validate_path_inside_base(&base, &output)?;
    if !src.exists() { return Err(EngineError::InvalidParameter("input".into(), "input file does not exist".into())); }
    let out = tokio::process::Command::new("ffmpeg").args(crate::recording::remux_args(&src, &dst)).output().await.map_err(|e| EngineError::Process(format!("spawn remux: {e}")))?;
    if !out.status.success() {
        return Err(EngineError::Process(format!("remux failed with exit {}", out.status.code().unwrap_or(-1))));
    }
    Ok(dst.to_string_lossy().into_owned())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_virtual_camera_status(state: tauri_state<'_, AppState>) -> EngineResult<VirtualCameraStatus> {
    let _ = &state;
    Ok(virtual_camera::probe())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn start_virtual_camera(state: tauri_state<'_, AppState>) -> EngineResult<VirtualCameraStatus> {
    let _ = &state;
    let probed = virtual_camera::probe();
    virtual_camera::ensure_startable(&probed)?;
    Err(EngineError::Internal("virtual-camera feed routing is not implemented yet — driver detected, output wiring pending".into()))
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn stop_virtual_camera(state: tauri_state<'_, AppState>) -> EngineResult<VirtualCameraStatus> {
    let _ = &state;
    Ok(virtual_camera::probe())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn get_settings(state: tauri_state<'_, AppState>) -> EngineResult<AppConfig> {
    Ok(state.settings.read().await.clone())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn save_settings(state: tauri_state<'_, AppState>, config: AppConfig, base_dir: String) -> EngineResult<String> {
    let json = config.to_json()?;
    let path = config_path_for_dir(&PathBuf::from(&base_dir));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| EngineError::Process(format!("create settings dir: {e}")))?;
    }
    std::fs::write(&path, json).map_err(|e| EngineError::Process(format!("write settings: {e}")))?;
    *state.settings.write().await = config;
    Ok(path.to_string_lossy().into_owned())
}
#[cfg_attr(feature = "tauri-app", tauri::command)]
pub async fn load_settings(state: tauri_state<'_, AppState>, base_dir: String) -> EngineResult<AppConfig> {
    let path = config_path_for_dir(&PathBuf::from(&base_dir));
    let cfg = match std::fs::read_to_string(&path) { Ok(json) => AppConfig::from_json(&json)?, Err(_) => AppConfig::default() };
    *state.settings.write().await = cfg.clone();
    Ok(cfg)
}
#[cfg(feature = "tauri-app")]
type tauri_state<'a, T> = tauri::State<'a, T>;
#[cfg(not(feature = "tauri-app"))]
#[allow(non_camel_case_types)]
type tauri_state<'a, T> = Arc<T>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::destinations::StreamMonitor;
    fn app() -> Arc<AppState> { Arc::new(AppState::new()) }
    #[tokio::test]
    async fn studio_rundown_smoke() {
        let a = app();
        let scene = create_scene(a.clone(), "Main".into()).await.unwrap();
        let src = add_source(a.clone(), scene.id.clone(), "Cam".into(), SourceKind::Camera).await.unwrap();
        assert!(!toggle_source(a.clone(), scene.id.clone(), src.id.clone()).await.unwrap());
        set_preview_scene(a.clone(), scene.id.clone()).await.unwrap();
        let live = take_transition(a.clone()).await.unwrap();
        assert_eq!(live, scene.id);
        delete_scene(a.clone(), scene.id.clone()).await.unwrap();
        assert!(list_scenes(a).await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn mixer_rejects_absurd_gain() {
        let a = app();
        assert!(set_channel_gain(a.clone(), ChannelId::Mic, 99.0).await.is_err());
        let mixer = set_channel_gain(a.clone(), ChannelId::Mic, -6.0).await.unwrap();
        assert_eq!(mixer.channels[0].gain_db, -6.0);
    }
    #[tokio::test]
    async fn stream_start_fails_closed_without_setup() {
        let a = app();
        assert!(start_stream(a.clone(), "dest-99".into(), CaptureInput::TestPattern).await.is_err());
        add_destination(a.clone(), Platform::Custom, "Lab".into(), "rtmp://127.0.0.1/live".into()).await.unwrap();
        assert!(start_stream(a.clone(), "dest-1".into(), CaptureInput::TestPattern).await.is_err());
        assert!(stop_stream(a.clone()).await.is_err());
        let _ = StreamMonitor::default();
    }
    #[tokio::test]
    async fn recording_rejects_bad_extension_before_spawning() {
        let a = app();
        let bad = RecordingConfig { file_name: "clip.mp4".into(), format: crate::recording::RecordingFormat::Mkv, tracks: crate::recording::TrackMode::Mixed, encoder: "libx264".into(), video_bitrate_kbps: 8000 };
        assert!(start_recording(a.clone(), bad, CaptureInput::TestPattern, "/tmp/hub-rec".into()).await.is_err());
        assert!(stop_recording(a).await.is_err());
    }
    #[tokio::test]
    async fn settings_save_load_roundtrip() {
        let a = app();
        let dir = std::env::temp_dir().join("msh-phase7-test");
        let path = save_settings(a.clone(), AppConfig::default(), dir.to_string_lossy().into_owned()).await.unwrap();
        assert!(path.ends_with("mystreamhub.config.json"));
        let loaded = load_settings(a.clone(), dir.to_string_lossy().into_owned()).await.unwrap();
        assert_eq!(loaded, AppConfig::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[tokio::test]
    async fn virtual_camera_start_refused_without_driver() {
        let a = app();
        let status = get_virtual_camera_status(a.clone()).await.unwrap();
        assert!(!status.active);
        assert!(start_virtual_camera(a).await.is_err());
    }
    #[cfg(not(feature = "tauri-app"))]
    #[tokio::test]
    async fn oauth_store_fails_closed_without_keyring() {
        let a = app();
        let token = OAuthToken { platform: Platform::Twitch, access_token: "abc".into(), refresh_token: None, expires_at: u64::MAX };
        let err = store_oauth_token(a.clone(), token).await.unwrap_err();
        assert!(matches!(err, EngineError::CredentialStore));
        assert!(!oauth_status(a, Platform::Twitch).await.unwrap());
    }
}
