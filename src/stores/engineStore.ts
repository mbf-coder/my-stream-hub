/** Phase-1 engine store: status, capabilities, encoders, devices, log. */
import { create } from 'zustand';
import { api } from '../services/tauri';
import type { AvailableEncoder, CaptureDevice, EngineStatus, MediaCapabilities } from '../types/engine';

interface EngineState {
  status: EngineStatus | null;
  capabilities: MediaCapabilities | null;
  encoders: AvailableEncoder[];
  audioDevices: CaptureDevice[];
  videoDevices: CaptureDevice[];
  busy: boolean;
  log: string[];
  error: string | null;
  refreshStatus: () => Promise<void>;
  initialize: () => Promise<void>;
  loadCapabilities: () => Promise<void>;
  loadEncoders: () => Promise<void>;
  loadDevices: () => Promise<void>;
  startTest: () => Promise<void>;
  stopTest: () => Promise<void>;
}

const initialStatus: EngineStatus = {
  state: 'UNINITIALIZED',
  ffmpeg_version: null,
  ffprobe_version: null,
  active_encoder: null,
  last_error: null,
};

function errMsg(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export const useEngineStore = create<EngineState>()((set, get) => {
  const pushLog = (line: string) =>
    set((s) => ({ log: [...s.log.slice(-199), `${new Date().toLocaleTimeString()} ${line}`] }));
  const guard = async (label: string, fn: () => Promise<void>) => {
    set({ busy: true, error: null });
    try {
      await fn();
      pushLog(label);
    } catch (e) {
      const message = errMsg(e);
      set({ error: message });
      pushLog(`${label} — FAILED: ${message}`);
    } finally {
      set({ busy: false });
    }
  };
  return {
    status: initialStatus,
    capabilities: null,
    encoders: [],
    audioDevices: [],
    videoDevices: [],
    busy: false,
    log: [],
    error: null,
    refreshStatus: async () => {
      const status = await api.getEngineStatus();
      set({ status });
    },
    initialize: () => guard('initialize_engine → ok', async () => {
      const status = await api.initializeEngine();
      set({ status });
      await get().loadCapabilities();
      await get().loadEncoders();
    }),
    loadCapabilities: () => guard('capabilities loaded', async () => {
      set({ capabilities: await api.getMediaCapabilities() });
    }),
    loadEncoders: () => guard('encoders loaded', async () => {
      set({ encoders: await api.getAvailableEncoders() });
    }),
    loadDevices: () => guard('devices loaded', async () => {
      const [audioDevices, videoDevices] = await Promise.all([
        api.getAudioDevices().catch(() => [] as CaptureDevice[]),
        api.getVideoDevices().catch(() => [] as CaptureDevice[]),
      ]);
      set({ audioDevices, videoDevices });
    }),
    startTest: () => guard('media test started', async () => {
      set({ status: await api.startMediaTest() });
    }),
    stopTest: () => guard('media test stopped', async () => {
      set({ status: await api.stopMediaTest() });
    }),
  };
});
