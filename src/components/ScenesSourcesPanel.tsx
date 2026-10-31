import { useState } from 'react';
import { Layers, Plus, Trash2, Eye, EyeOff } from 'lucide-react';
import { useStudioStore } from '../stores/studioStore';
import type { SourceKind } from '../types/engine';

const KINDS: SourceKind[] = ['display', 'window', 'camera', 'image', 'text', 'media'];

/** Phase-2 scene rundown + per-scene source list. */
export default function ScenesSourcesPanel() {
  const s = useStudioStore();
  const [newScene, setNewScene] = useState('');
  const [newSource, setNewSource] = useState('');
  const [newKind, setNewKind] = useState<SourceKind>('camera');
  const preview = s.scenes.find((sc) => sc.id === s.previewId);
  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4" aria-label="Scenes and sources">
      <div className="flex items-center gap-2 mb-3">
        <Layers size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">SCENES &amp; SOURCES</h2>
      </div>
      <div className="flex gap-1 mb-2">
        <input value={newScene} onChange={(e) => setNewScene(e.target.value)} placeholder="New scene name"
          className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1" />
        <button onClick={() => { void s.createScene(newScene); setNewScene(''); }} disabled={s.busy || !newScene.trim()}
          className="rounded bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 px-2 py-1" aria-label="Add scene">
          <Plus size={14} />
        </button>
      </div>
      <ul className="space-y-1 mb-3 max-h-36 overflow-y-auto">
        {s.scenes.map((scene) => (
          <li key={scene.id} className="flex items-center gap-1 text-xs">
            <button onClick={() => void s.preview(scene.id)} title="Preview"
              className={`flex-1 text-left truncate rounded px-2 py-1 ${s.previewId === scene.id ? 'bg-emerald-900 text-emerald-200' : 'bg-slate-800 hover:bg-slate-700'}`}>
              {scene.name} <span className="text-slate-500">({scene.sources.length})</span>
            </button>
            <button onClick={() => void s.program(scene.id)} title="Send to program"
              className={`rounded px-1.5 py-1 ${s.programId === scene.id ? 'bg-red-900 text-red-200' : 'bg-slate-800 hover:bg-slate-700'}`}>PGM</button>
            <button onClick={() => void s.deleteScene(scene.id)} title="Delete scene"
              className="rounded bg-slate-800 hover:bg-red-900 px-1.5 py-1"><Trash2 size={12} /></button>
          </li>
        ))}
        {s.scenes.length === 0 && <p className="text-xs text-slate-500">No scenes yet.</p>}
      </ul>
      {preview && (
        <div className="border-t border-studio-border pt-2">
          <p className="text-[11px] text-slate-400 mb-1">Sources in “{preview.name}”</p>
          <div className="flex gap-1 mb-2">
            <input value={newSource} onChange={(e) => setNewSource(e.target.value)} placeholder="Source name"
              className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1" />
            <select value={newKind} onChange={(e) => setNewKind(e.target.value as SourceKind)}
              className="rounded bg-slate-800 border border-studio-border text-xs px-1 py-1">
              {KINDS.map((k) => <option key={k} value={k}>{k}</option>)}
            </select>
            <button onClick={() => { void s.addSource(preview.id, newSource, newKind); setNewSource(''); }}
              disabled={s.busy || !newSource.trim()}
              className="rounded bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 px-2 py-1" aria-label="Add source">
              <Plus size={14} />
            </button>
          </div>
          <ul className="space-y-1 max-h-32 overflow-y-auto">
            {preview.sources.map((src, i) => (
              <li key={src.id} className="flex items-center gap-1 text-xs bg-slate-800/60 rounded px-2 py-1">
                <span className="text-slate-500 font-mono">{i + 1}</span>
                <span className="flex-1 truncate">{src.name} <span className="text-slate-500">· {src.kind}</span></span>
                <button onClick={() => void s.toggleSource(preview.id, src.id)} title="Toggle visibility" className="px-1">
                  {src.visible ? <Eye size={12} className="text-emerald-400" /> : <EyeOff size={12} className="text-slate-500" />}
                </button>
                <button onClick={() => void s.removeSource(preview.id, src.id)} title="Remove source" className="px-1 hover:text-red-400"><Trash2 size={12} /></button>
              </li>
            ))}
          </ul>
        </div>
      )}
      {s.error && <p className="text-[11px] text-red-300 mt-2 break-words">{s.error}</p>}
    </section>
  );
}
