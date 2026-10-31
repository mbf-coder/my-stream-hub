//! MY STREAM HUB — library root (Phase 1).
//!
//! Module layout mirrors the spec §4 `src-tauri/src` tree. The Tauri shell
//! (`run_app`) only exists in default-feature builds; the media core compiles
//! everywhere so Linux CI can execute the full unit suite.

pub mod audio;
pub mod commands;
pub mod compositor;
pub mod configuration;
pub mod devices;
pub mod errors;
pub mod events;
pub mod media;
pub mod monitoring;
pub mod recording;
pub mod scenes;
pub mod security;
pub mod state;
pub mod streaming;
pub mod virtual_camera;

/// Boot the Tauri 2 runtime (Windows 11 target).
#[cfg(feature = "tauri-app")]
pub fn run_app() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "my_stream_hub=debug,tauri=info",
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = commands::AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_engine_status,
            commands::initialize_engine,
            commands::get_media_capabilities,
            commands::get_available_encoders,
            commands::get_audio_devices,
            commands::get_video_devices,
            commands::start_media_test,
            commands::stop_media_test,
            commands::list_scenes,
            commands::create_scene,
            commands::delete_scene,
            commands::rename_scene,
            commands::add_source,
            commands::remove_source,
            commands::toggle_source,
            commands::move_source,
            commands::set_preview_scene,
            commands::set_program_scene,
            commands::take_transition,
            commands::set_transition,
            commands::build_compositor_filter,
            commands::get_mixer,
            commands::set_channel_gain,
            commands::set_channel_mute,
            commands::set_master_gain,
            commands::set_master_mute,
            commands::set_eq,
            commands::set_voice_fx,
            commands::scan_vst3,
            commands::list_destinations,
            commands::add_destination,
            commands::remove_destination,
            commands::toggle_destination,
            commands::set_stream_key,
            commands::store_oauth_token,
            commands::oauth_status,
            commands::start_stream,
            commands::stop_stream,
            commands::get_stream_status,
            commands::start_recording,
            commands::stop_recording,
            commands::get_recording_status,
            commands::remux_recording,
            commands::get_virtual_camera_status,
            commands::start_virtual_camera,
            commands::stop_virtual_camera,
            commands::get_settings,
            commands::save_settings,
            commands::load_settings,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run MY STREAM HUB");
}
