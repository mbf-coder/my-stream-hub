/** Phase-2 studio rundown store: scenes, sources, preview/program, transition. */
import { create } from 'zustand';
import { api } from '../services/tauri';
import type { Scene, SourceKind, Transition } from '../types/engine';

interface StudioState {
  scenes: Scene[];
  previewId: string | null;
  programId: string | null;
  transition: Transition;
  busy: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  createScene: (name: string) => Promise<void>;
  deleteScene: (id: string) => Promise<void>;
  renameScene: (id: string, name: string) => Promise<void>;
  addSource: (sceneId: string, name: string, kind: SourceKind) => Promise<void>;
  removeSource: (sceneId: string, sourceId: string) => Promise<void>;
  toggleSource: (sceneId: string, sourceId: string) => Promise<void>;
  preview: (id: string) => Promise<void>;
  program: (id: string) => Promise<void>;
  take: () => Promise<void>;
  setTransition: (t: Transition) => Promise<void>;
}

function errMsg(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export const useStudioStore = create<StudioState>()((set) => {
  const guard = async (fn: () => Promise<Partial<StudioState>>) => {
    set({ busy: true, error: null });
    try {
      set({ ...(await fn()), busy: false });
    } catch (e) {
      set({ error: errMsg(e), busy: false });
    }
  };
  const reload = async (): Promise<Partial<StudioState>> => {
    const scenes = await api.listScenes();
    return { scenes };
  };
  return {
    scenes: [],
    previewId: null,
    programId: null,
    transition: { kind: 'cut', duration_ms: 300 },
    busy: false,
    error: null,
    refresh: () => guard(reload),
    createScene: (name) => guard(async () => {
      const scene = await api.createScene(name);
      const scenes = await api.listScenes();
      return { scenes, previewId: scene.id };
    }),
    deleteScene: (id) => guard(async () => {
      await api.deleteScene(id);
      return reload();
    }),
    renameScene: (id, name) => guard(async () => {
      await api.renameScene(id, name);
      return reload();
    }),
    addSource: (sceneId, name, kind) => guard(async () => {
      await api.addSource(sceneId, name, kind);
      return reload();
    }),
    removeSource: (sceneId, sourceId) => guard(async () => {
      await api.removeSource(sceneId, sourceId);
      return reload();
    }),
    toggleSource: (sceneId, sourceId) => guard(async () => {
      await api.toggleSource(sceneId, sourceId);
      return reload();
    }),
    preview: (id) => guard(async () => {
      await api.setPreviewScene(id);
      return { previewId: id };
    }),
    program: (id) => guard(async () => {
      await api.setProgramScene(id);
      return { programId: id };
    }),
    take: () => guard(async () => {
      const programId = await api.takeTransition();
      return { programId };
    }),
    setTransition: (t) => guard(async () => ({ transition: await api.setTransition(t) })),
  };
});
