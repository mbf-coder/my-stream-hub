//! IPC contract: the frontend may only invoke these 49 commands (Phases 1–7).
//! Mirrors `src/types/engine.ts` IPC_COMMANDS — both lists must match exactly.

pub const EXPECTED_IPC_COMMANDS: &[&str] = &[
    // Phase 1 — core (8)
    "get_engine_status",
    "initialize_engine",
    "get_media_capabilities",
    "get_available_encoders",
    "get_audio_devices",
    "get_video_devices",
    "start_media_test",
    "stop_media_test",
    // Phase 2 — studio rundown (12)
    "list_scenes",
    "create_scene",
    "delete_scene",
    "rename_scene",
    "add_source",
    "remove_source",
    "toggle_source",
    "move_source",
    "set_preview_scene",
    "set_program_scene",
    "take_transition",
    "set_transition",
    // Phase 3 — compositing (1)
    "build_compositor_filter",
    // Phase 4 — audio (8)
    "get_mixer",
    "set_channel_gain",
    "set_channel_mute",
    "set_master_gain",
    "set_master_mute",
    "set_eq",
    "set_voice_fx",
    "scan_vst3",
    // Phase 5 — streaming (10)
    "list_destinations",
    "add_destination",
    "remove_destination",
    "toggle_destination",
    "set_stream_key",
    "store_oauth_token",
    "oauth_status",
    "start_stream",
    "stop_stream",
    "get_stream_status",
    // Phase 6 — recording + virtual camera (7)
    "start_recording",
    "stop_recording",
    "get_recording_status",
    "remux_recording",
    "get_virtual_camera_status",
    "start_virtual_camera",
    "stop_virtual_camera",
    // Phase 7 — settings (3)
    "get_settings",
    "save_settings",
    "load_settings",
];

#[test]
fn command_set_matches_spec() {
    assert_eq!(EXPECTED_IPC_COMMANDS.len(), 49);
    let mut sorted = EXPECTED_IPC_COMMANDS.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        EXPECTED_IPC_COMMANDS.len(),
        "duplicate command names"
    );
    for phase1 in [
        "get_engine_status",
        "initialize_engine",
        "get_media_capabilities",
        "get_available_encoders",
        "get_audio_devices",
        "get_video_devices",
        "start_media_test",
        "stop_media_test",
    ] {
        assert!(EXPECTED_IPC_COMMANDS.contains(&phase1));
    }
}
