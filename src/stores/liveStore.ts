/** Phase 4–6 live store: mixer, destinations, stream, recording, virtual camera. */
import { create } from 'zustand';
import { api } from '../services/tauri';
import type {
  CaptureInput,
  ChannelId,
  Destination,
  Eq3,
  Mixer,
  Platform,
  RecordStatusPayload,
  StreamStatusPayload,
  VirtualCameraStatus,
  VoiceFx,
} from '../types/engine';
import type { RecordingConfig } from '../services/stream';

interface LiveState {
  mixer: Mixer | null;
  destinations: Destination[];
  stream: StreamStatusPayload | null;
  recording: RecordStatusPayload | null;
  vc: VirtualCameraStatus | null;
  busy: boolean;
  error: string | null;
  refreshMixer: () => Promise<void>;
  setGain: (channel: ChannelId, gain_db: number) => Promise<void>;
  setMute: (channel: ChannelId, muted: boolean) => Promise<void>;
  setMasterGain: (gain_db: number) => Promise<void>;
  setMasterMute: (muted: boolean) => Promise<void>;
  setEq: (eq: Eq3) => Promise<void>;
  setVoiceFx: (fx: VoiceFx) => Promise<void>;
  refreshDestinations: () => Promise<void>;
  addDestination: (platform: Platform, label: string, url: string) => Promise<void>;
  removeDestination: (id: string) => Promise<void>;
  toggleDestination: (id: string) => Promise<void>;
  saveStreamKey: (key: string) => Promise<boolean>;
  startStream: (destination_id: string, testPattern: boolean) => Promise<void>;
  stopStream: () => Promise<void>;
  refreshStream: () => Promise<void>;
  startRecording: (config: RecordingConfig, base_dir: string) => Promise<void>;
  stopRecording: () => Promise<void>;
  refreshRecording: () => Promise<void>;
  remuxResult: string | null;
  remux: (input: string, output: string, base_dir: string) => Promise<void>;
  refreshVc: () => Promise<void>;
  startVc: () => Promise<void>;
  stopVc: () => Promise<void>;
}

function errMsg(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

const TEST_INPUT: CaptureInput = 'TestPattern';

export const useLiveStore = create<LiveState>()((set) => {
  const guard = async (fn: () => Promise<void>) => {
    set({ busy: true, error: null });
    try {
      await fn();
    } catch (e) {
      set({ error: errMsg(e) });
    } finally {
      set({ busy: false });
    }
  };

  return {
    mixer: null,
    destinations: [],
    stream: null,
    recording: null,
    vc: null,
    busy: false,
    error: null,

    refreshMixer: () => guard(async () => set({ mixer: await api.getMixer() })),
    setGain: (channel, gain_db) =>
      guard(async () => set({ mixer: await api.setChannelGain(channel, gain_db) })),
    setMute: (channel, muted) =>
      guard(async () => set({ mixer: await api.setChannelMute(channel, muted) })),
    setMasterGain: (gain_db) =>
      guard(async () => set({ mixer: await api.setMasterGain(gain_db) })),
    setMasterMute: (muted) =>
      guard(async () => set({ mixer: await api.setMasterMute(muted) })),
    setEq: (eq) => guard(async () => set({ mixer: await api.setEq(eq) })),
    setVoiceFx: (fx) => guard(async () => set({ mixer: await api.setVoiceFx(fx) })),

    refreshDestinations: () =>
      guard(async () => set({ destinations: await api.listDestinations() })),
    addDestination: (platform, label, url) =>
      guard(async () => {
        await api.addDestination(platform, label, url);
        set({ destinations: await api.listDestinations() });
      }),
    removeDestination: (id) =>
      guard(async () => {
        await api.removeDestination(id);
        set({ destinations: await api.listDestinations() });
      }),
    toggleDestination: (id) =>
      guard(async () => {
        await api.toggleDestination(id);
        set({ destinations: await api.listDestinations() });
      }),
    saveStreamKey: async (key) => {
      try {
        await api.setStreamKey(key);
        set({ error: null });
        return true;
      } catch (e) {
        set({ error: errMsg(e) });
        return false;
      }
    },
    startStream: (destination_id, testPattern) =>
      guard(async () => {
        await api.startStream(destination_id, testPattern ? TEST_INPUT : TEST_INPUT);
        set({ stream: await api.getStreamStatus() });
      }),
    stopStream: () =>
      guard(async () => {
        await api.stopStream();
        set({ stream: await api.getStreamStatus() });
      }),
    refreshStream: () => guard(async () => set({ stream: await api.getStreamStatus() })),

    startRecording: (config, base_dir) =>
      guard(async () => {
        await api.startRecording(config, TEST_INPUT, base_dir);
        set({ recording: await api.getRecordingStatus() });
      }),
    stopRecording: () =>
      guard(async () => {
        await api.stopRecording();
        set({ recording: await api.getRecordingStatus() });
      }),
    refreshRecording: () =>
      guard(async () => set({ recording: await api.getRecordingStatus() })),
    remuxResult: null,
    remux: (input, output, base_dir) =>
      guard(async () => set({ remuxResult: await api.remuxRecording(input, output, base_dir) })),
    refreshVc: () => guard(async () => set({ vc: await api.getVirtualCameraStatus() })),
    startVc: () => guard(async () => set({ vc: await api.startVirtualCamera() })),
    stopVc: () => guard(async () => set({ vc: await api.stopVirtualCamera() })),
  };
});
