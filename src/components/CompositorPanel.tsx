import { useState } from 'react';
import { Layers } from 'lucide-react';
import { api } from '../services/tauri';
import type { CanvasPreset, Layer, Transform } from '../types/engine';

/** Phase-3 compositing editor: canvas preset + per-layer transforms → validated `filter_complex`. */

export function defaultTransform(): Transform {
  return { x: 0, y: 0, width: 1920, height: 1080, rotation_deg: 0, crop: [0, 0, 0, 0] };
}

export function defaultLayer(input_index: number): Layer {
  return { input_index, transform: defaultTransform() };
}

/** Clamp u32-bound fields to >= 0 integers so the IPC payload is well-formed
 * (semantic validation — ranges, canvas fit — stays in the Rust builder). */
export function sanitizeLayer(layer: Layer): Layer {
  const num = (v: number) => (Number.isFinite(v) ? Math.max(0, Math.floor(v)) : 0);
  const t = layer.transform;
  return {
    input_index: num(layer.input_index),
    transform: {
      x: Math.trunc(Number.isFinite(t.x) ? t.x : 0),
      y: Math.trunc(Number.isFinite(t.y) ? t.y : 0),
      width: Math.max(1, num(t.width)),
      height: Math.max(1, num(t.height)),
      rotation_deg: Math.trunc(Number.isFinite(t.rotation_deg) ? t.rotation_deg : 0),
      crop: [num(t.crop[0]), num(t.crop[1]), num(t.crop[2]), num(t.crop[3])],
    },
  };
}

const CANVASES: CanvasPreset[] = ['p1080', 'p720', 'p4k'];

export default function CompositorPanel() {
  const [canvas, setCanvas] = useState<CanvasPreset>('p1080');
  const [layers, setLayers] = useState<Layer[]>([defaultLayer(0)]);
  const [filter, setFilter] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const patchTransform = (i: number, key: keyof Transform, value: number) => {
    setLayers((ls) => ls.map((l, j) => (j === i ? { ...l, transform: { ...l.transform, [key]: value } } : l)));
  };

  const patchCrop = (i: number, k: number, value: number) => {
    setLayers((ls) =>
      ls.map((l, j) => {
        if (j !== i) return l;
        const crop = [...l.transform.crop] as [number, number, number, number];
        crop[k] = value;
        return { ...l, transform: { ...l.transform, crop } };
      }),
    );
  };

  const build = async () => {
    setBusy(true);
    setError(null);
    setFilter(null);
    try {
      const out = await api.buildCompositorFilter(canvas, layers.map(sanitizeLayer));
      setFilter(out);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4 md:col-span-2" aria-label="Compositor">
      <div className="flex items-center gap-2 mb-3">
        <Layers size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">COMPOSITOR</h2>
        <select
          value={canvas}
          onChange={(e) => setCanvas(e.target.value as CanvasPreset)}
          className="ml-auto rounded bg-slate-800 border border-studio-border text-xs px-2 py-1"
          aria-label="Canvas preset"
        >
          {CANVASES.map((c) => (
            <option key={c} value={c}>{c}</option>
          ))}
        </select>
      </div>
      <div className="space-y-2 mb-3">
        {layers.map((l, i) => (
          <div key={i} className="rounded bg-slate-800/60 px-2 py-2 text-xs">
            <div className="flex items-center gap-2 mb-1">
              <span className="font-semibold text-slate-300">Layer {i}{i === 0 ? ' (base)' : ''}</span>
              <label className="text-slate-500">input
                <input
                  type="number" min={0} value={l.input_index}
                  onChange={(e) => setLayers((ls) => ls.map((x, j) => (j === i ? { ...x, input_index: Number(e.target.value) } : x)))}
                  className="ml-1 w-14 rounded bg-slate-900 border border-studio-border px-1 py-0.5"
                />
              </label>
              {layers.length > 1 && (
                <button onClick={() => setLayers((ls) => ls.filter((_, j) => j !== i))} className="ml-auto text-slate-500 hover:text-red-400 text-[10px]">✕</button>
              )}
            </div>
            <div className="grid grid-cols-4 gap-1">
              {(['x', 'y', 'width', 'height'] as const).map((k) => (
                <label key={k} className="text-slate-500">{k}
                  <input
                    type="number" value={l.transform[k]}
                    onChange={(e) => patchTransform(i, k, Number(e.target.value))}
                    className="ml-1 w-full min-w-0 rounded bg-slate-900 border border-studio-border px-1 py-0.5"
                  />
                </label>
              ))}
            </div>
            <div className="grid grid-cols-5 gap-1 mt-1">
              <label className="text-slate-500">rot°
                <input
                  type="number" value={l.transform.rotation_deg}
                  onChange={(e) => patchTransform(i, 'rotation_deg', Number(e.target.value))}
                  className="ml-1 w-full min-w-0 rounded bg-slate-900 border border-studio-border px-1 py-0.5"
                />
              </label>
              {(['l', 't', 'r', 'b'] as const).map((k, kk) => (
                <label key={k} className="text-slate-500">crop {k}
                  <input
                    type="number" min={0} value={l.transform.crop[kk]}
                    onChange={(e) => patchCrop(i, kk, Number(e.target.value))}
                    className="ml-1 w-full min-w-0 rounded bg-slate-900 border border-studio-border px-1 py-0.5"
                  />
                </label>
              ))}
            </div>
          </div>
        ))}
      </div>
      <div className="flex gap-2 items-center">
        <button
          onClick={() => setLayers((ls) => (ls.length >= 16 ? ls : [...ls, defaultLayer(ls.length)]))}
          disabled={busy || layers.length >= 16}
          className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-2 py-1 text-xs"
        >
          + Add layer
        </button>
        <button
          onClick={() => void build()}
          disabled={busy || layers.length === 0}
          className="rounded bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 px-3 py-1 text-xs font-bold"
        >
          {busy ? 'Building…' : 'Build filter'}
        </button>
      </div>
      {filter && <pre className="text-[11px] text-emerald-300 mt-2 font-mono break-all whitespace-pre-wrap">{filter}</pre>}
      {error && <p className="text-[11px] text-red-300 mt-2 break-words">{error}</p>}
    </section>
  );
}
