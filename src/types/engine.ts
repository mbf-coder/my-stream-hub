/** Mirror of the Rust IPC contracts (serde). Keep in sync with src-tauri. */

export type EngineState =
  | 'UNINITIALIZED' | 'INITIALIZING' | 'READY' | 'RUNNING'
  | 'STOPPING' | 'STOPPED' | 'ERROR';

export interface EngineStatus {
  state: EngineState;
  ffmpeg_version: string | null;
  ffprobe_version: string | null;
  active_encoder: string | null;
  last_error: string | null;
}

export interface MediaCapabilities {
  ffmpeg_version: string | null;
  ffprobe_version: string | null;
  has_libx264: boolean;
  has_aac: boolean;
  rtmp_supported: boolean;
}

export type EncoderKind = 'nvenc_h264' | 'amf_h264' | 'qsv_h264' | 'libx264';

export interface AvailableEncoder {
  kind: EncoderKind;
  name: string;
  hardware: boolean;
}

export interface CaptureDevice {
  name: string;
  kind: 'audio' | 'video';
  detail: string;
}

export type SourceKind = 'display' | 'window' | 'camera' | 'image' | 'text' | 'media';

export interface StudioSource {
  id: string;
  name: string;
  kind: SourceKind;
  visible: boolean;
  volume_db: number;
}

export interface Scene {
  id: string;
  name: string;
  sources: StudioSource[];
}

export type TransitionKind = 'cut' | 'fade';

export interface Transition {
  kind: TransitionKind;
  duration_ms: number;
}

export type CanvasPreset = 'p1080' | 'p720' | 'p4k';

export interface Transform {
  x: number;
  y: number;
  width: number;
  height: number;
  rotation_deg: number;
  crop: [number, number, number, number];
}

export interface Layer {
  input_index: number;
  transform: Transform;
}

export type ChannelId = 'mic' | 'desktop' | 'aux';

export interface EqBand {
  freq_hz: number;
  gain_db: number;
  q: number;
}

export interface Eq3 {
  low: EqBand;
  mid: EqBand;
  high: EqBand;
}

export type VoiceFx = 'none' | { pitch_shift: number } | 'robot' | 'echo';

export interface Channel {
  id: ChannelId;
  gain_db: number;
  muted: boolean;
  peak: number;
}

export interface Mixer {
  channels: Channel[];
  master_gain_db: number;
  master_muted: boolean;
  eq: Eq3;
  voice_fx: VoiceFx;
  monitor_muted: ChannelId[];
}

export interface VstPlugin {
  name: string;
  path: string;
  enabled: boolean;
}

export type Platform = 'twitch' | 'youTube' | 'kick' | 'custom';

export interface Destination {
  id: string;
  platform: Platform;
  label: string;
  url: string;
  enabled: boolean;
}

export interface OAuthToken {
  platform: Platform;
  access_token: string;
  refresh_token: string | null;
  expires_at: number;
}

/** Serde external tagging: `"TestPattern"` or `{ "Device": {...} }`. */
export type CaptureInput = 'TestPattern' | { Device: { video: string; audio?: string | null } };

export interface StreamStatusPayload {
  live: boolean;
  destination_label: string | null;
  bitrate_kbps: number;
  dropped_frames: number;
  uptime_secs: number;
}

export interface RecordStatusPayload {
  recording: boolean;
  path: string | null;
}

export interface VirtualCameraStatus {
  driver_installed: boolean;
  driver_detail: string | null;
  active: boolean;
}

export interface AppSettings {
  version: number;
  language: string;
  theme: 'dark' | 'light';
  base_width: number;
  base_height: number;
  output_width: number;
  output_height: number;
  fps: number;
  sample_rate: number;
  channels: number;
  stream: { encoder: string; video_bitrate_kbps: number; audio_bitrate_kbps: number };
  recording: { format: string; directory: string; video_bitrate_kbps: number };
  transition_kind: string;
  transition_ms: number;
}

/** All 49 Phase 1–7 IPC commands — single source of truth for invoke(). */
export const IPC_COMMANDS = [
  'get_engine_status', 'initialize_engine', 'get_media_capabilities',
  'get_available_encoders', 'get_audio_devices', 'get_video_devices',
  'start_media_test', 'stop_media_test',
  'list_scenes', 'create_scene', 'delete_scene', 'rename_scene',
  'add_source', 'remove_source', 'toggle_source', 'move_source',
  'set_preview_scene', 'set_program_scene', 'take_transition', 'set_transition',
  'build_compositor_filter',
  'get_mixer', 'set_channel_gain', 'set_channel_mute', 'set_master_gain',
  'set_master_mute', 'set_eq', 'set_voice_fx', 'scan_vst3',
  'list_destinations', 'add_destination', 'remove_destination', 'toggle_destination',
  'set_stream_key', 'store_oauth_token', 'oauth_status', 'start_stream',
  'stop_stream', 'get_stream_status',
  'start_recording', 'stop_recording', 'get_recording_status', 'remux_recording',
  'get_virtual_camera_status', 'start_virtual_camera', 'stop_virtual_camera',
  'get_settings', 'save_settings', 'load_settings',
] as const;

export type IpcCommand = (typeof IPC_COMMANDS)[number];
