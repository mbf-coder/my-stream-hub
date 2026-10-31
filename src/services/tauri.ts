/**
 * IPC bridge. Inside the Tauri WebView it uses `@tauri-apps/api/core::invoke`;
 * in a plain browser (vite dev without Tauri) it throws a typed error,
 * so UI work never depends on the Rust shell.
 */
import type {
  AppSettings, AvailableEncoder, CanvasPreset, CaptureDevice, CaptureInput,
  ChannelId, Destination, EngineStatus, Eq3, IpcCommand, Layer,
  MediaCapabilities, Mixer, OAuthToken, Platform, RecordStatusPayload,
  Scene, SourceKind, StudioSource, StreamStatusPayload, Transition,
  VirtualCameraStatus, VoiceFx, VstPlugin,
} from '../types/engine';
import type { RecordingConfig } from './stream';

async function tauriInvoke<T>(cmd: IpcCommand, args?: Record<string, unknown>): Promise<T> {
  const w = window as unknown as { __TAURI__?: unknown };
  if (w.__TAURI__ !== undefined) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(cmd, args);
  }
  throw new Error(`Tauri runtime unavailable (browser preview). Command '${cmd}' requires the desktop shell.`);
}

export const api = {
  getEngineStatus: () => tauriInvoke<EngineStatus>('get_engine_status'),
  initializeEngine: () => tauriInvoke<EngineStatus>('initialize_engine'),
  getMediaCapabilities: () => tauriInvoke<MediaCapabilities>('get_media_capabilities'),
  getAvailableEncoders: () => tauriInvoke<AvailableEncoder[]>('get_available_encoders'),
  getAudioDevices: () => tauriInvoke<CaptureDevice[]>('get_audio_devices'),
  getVideoDevices: () => tauriInvoke<CaptureDevice[]>('get_video_devices'),
  startMediaTest: () => tauriInvoke<EngineStatus>('start_media_test'),
  stopMediaTest: () => tauriInvoke<EngineStatus>('stop_media_test'),
  listScenes: () => tauriInvoke<Scene[]>('list_scenes'),
  createScene: (name: string) => tauriInvoke<Scene>('create_scene', { name }),
  deleteScene: (scene_id: string) => tauriInvoke<void>('delete_scene', { scene_id }),
  renameScene: (scene_id: string, name: string) => tauriInvoke<Scene>('rename_scene', { scene_id, name }),
  addSource: (scene_id: string, name: string, kind: SourceKind) =>
    tauriInvoke<StudioSource>('add_source', { scene_id, name, kind }),
  removeSource: (scene_id: string, source_id: string) =>
    tauriInvoke<void>('remove_source', { scene_id, source_id }),
  toggleSource: (scene_id: string, source_id: string) =>
    tauriInvoke<boolean>('toggle_source', { scene_id, source_id }),
  moveSource: (scene_id: string, source_id: string, to_index: number) =>
    tauriInvoke<StudioSource[]>('move_source', { scene_id, source_id, to_index }),
  setPreviewScene: (scene_id: string) => tauriInvoke<void>('set_preview_scene', { scene_id }),
  setProgramScene: (scene_id: string) => tauriInvoke<void>('set_program_scene', { scene_id }),
  takeTransition: () => tauriInvoke<string>('take_transition'),
  setTransition: (transition: Transition) => tauriInvoke<Transition>('set_transition', { transition }),
  buildCompositorFilter: (canvas: CanvasPreset, layers: Layer[]) =>
    tauriInvoke<string>('build_compositor_filter', { canvas, layers }),
  getMixer: () => tauriInvoke<Mixer>('get_mixer'),
  setChannelGain: (channel: ChannelId, gain_db: number) =>
    tauriInvoke<Mixer>('set_channel_gain', { channel, gain_db }),
  setChannelMute: (channel: ChannelId, muted: boolean) =>
    tauriInvoke<Mixer>('set_channel_mute', { channel, muted }),
  setMasterGain: (gain_db: number) => tauriInvoke<Mixer>('set_master_gain', { gain_db }),
  setMasterMute: (muted: boolean) => tauriInvoke<Mixer>('set_master_mute', { muted }),
  setEq: (eq: Eq3) => tauriInvoke<Mixer>('set_eq', { eq }),
  setVoiceFx: (fx: VoiceFx) => tauriInvoke<Mixer>('set_voice_fx', { fx }),
  scanVst3: (dir: string) => tauriInvoke<[VstPlugin[], string]>('scan_vst3', { dir }),
  listDestinations: () => tauriInvoke<Destination[]>('list_destinations'),
  addDestination: (platform: Platform, label: string, url: string) =>
    tauriInvoke<Destination>('add_destination', { platform, label, url }),
  removeDestination: (destination_id: string) =>
    tauriInvoke<void>('remove_destination', { destination_id }),
  toggleDestination: (destination_id: string) =>
    tauriInvoke<boolean>('toggle_destination', { destination_id }),
  setStreamKey: (key: string) => tauriInvoke<boolean>('set_stream_key', { key }),
  storeOAuthToken: (token: OAuthToken) => tauriInvoke<boolean>('store_oauth_token', { token }),
  oauthStatus: (platform: Platform) => tauriInvoke<boolean>('oauth_status', { platform }),
  startStream: (destination_id: string, input: CaptureInput) =>
    tauriInvoke<EngineStatus>('start_stream', { destination_id, input }),
  stopStream: () => tauriInvoke<EngineStatus>('stop_stream'),
  getStreamStatus: () => tauriInvoke<StreamStatusPayload>('get_stream_status'),
  startRecording: (config: RecordingConfig, input: CaptureInput, base_dir: string) =>
    tauriInvoke<EngineStatus>('start_recording', { config, input, base_dir }),
  stopRecording: () => tauriInvoke<EngineStatus>('stop_recording'),
  getRecordingStatus: () => tauriInvoke<RecordStatusPayload>('get_recording_status'),
  remuxRecording: (input: string, output: string, base_dir: string) =>
    tauriInvoke<string>('remux_recording', { input, output, base_dir }),
  getVirtualCameraStatus: () => tauriInvoke<VirtualCameraStatus>('get_virtual_camera_status'),
  startVirtualCamera: () => tauriInvoke<VirtualCameraStatus>('start_virtual_camera'),
  stopVirtualCamera: () => tauriInvoke<VirtualCameraStatus>('stop_virtual_camera'),
  getSettings: () => tauriInvoke<AppSettings>('get_settings'),
  saveSettings: (config: AppSettings, base_dir: string) =>
    tauriInvoke<string>('save_settings', { config, base_dir }),
  loadSettings: (base_dir: string) => tauriInvoke<AppSettings>('load_settings', { base_dir }),
};

export function isTauriRuntime(): boolean {
  return (window as unknown as { __TAURI__?: unknown }).__TAURI__ !== undefined;
}
