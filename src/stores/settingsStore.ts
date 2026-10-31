/** Phase-7 settings store: load / edit / save the full AppConfig mirror. */
import { create } from 'zustand';
import { api } from '../services/tauri';
import type { AppSettings } from '../types/engine';

interface SettingsState {
  settings: AppSettings | null;
  savedPath: string | null;
  busy: boolean;
  error: string | null;
  load: (baseDir: string) => Promise<void>;
  save: (baseDir: string) => Promise<void>;
  patch: (p: Partial<AppSettings>) => void;
}

function errMsg(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export const useSettingsStore = create<SettingsState>()((set, get) => ({
  settings: null,
  savedPath: null,
  busy: false,
  error: null,
  load: async (baseDir) => {
    set({ busy: true, error: null });
    try { set({ settings: await api.loadSettings(baseDir), busy: false }); }
    catch (e) { set({ error: errMsg(e), busy: false }); }
  },
  save: async (baseDir) => {
    const { settings } = get();
    if (!settings) return;
    set({ busy: true, error: null });
    try {
      const savedPath = await api.saveSettings(settings, baseDir);
      set({ savedPath, busy: false });
    } catch (e) { set({ error: errMsg(e), busy: false }); }
  },
  patch: (p) => set((s) => (s.settings ? { settings: { ...s.settings, ...p } } : {})),
}));
