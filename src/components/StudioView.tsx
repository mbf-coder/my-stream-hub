import { ArrowRight, Clapperboard } from 'lucide-react';
import { useStudioStore } from '../stores/studioStore';

/** Phase-2 preview/program buses + TAKE bar. */
export default function StudioView() {
  const { scenes, previewId, programId, transition, take, setTransition, busy } = useStudioStore();
  const preview = scenes.find((s) => s.id === previewId);
  const program = scenes.find((s) => s.id === programId);
  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4 md:col-span-2" aria-label="Preview and program">
      <div className="flex items-center gap-2 mb-3">
        <Clapperboard size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">PREVIEW → PROGRAM</h2>
      </div>
      <div className="grid grid-cols-[1fr_auto_1fr] gap-3 items-stretch">
        <BusCard label="PREVIEW" name={preview?.name} count={preview?.sources.length ?? 0} accent="border-emerald-500" />
        <div className="flex flex-col justify-center gap-2">
          <button onClick={() => void take()} disabled={busy || !previewId}
            className="rounded bg-red-600 hover:bg-red-500 disabled:opacity-50 px-4 py-2 text-sm font-bold flex items-center gap-1">
            TAKE <ArrowRight size={14} />
          </button>
          <select value={transition.kind} disabled={busy}
            onChange={(e) => void setTransition({ kind: e.target.value as 'cut' | 'fade', duration_ms: transition.duration_ms })}
            className="rounded bg-slate-800 border border-studio-border text-xs px-2 py-1" aria-label="Transition kind">
            <option value="cut">CUT</option>
            <option value="fade">FADE</option>
          </select>
          {transition.kind === 'fade' && (
            <input type="number" min={50} max={5000} step={50} value={transition.duration_ms} disabled={busy}
              onChange={(e) => void setTransition({ kind: 'fade', duration_ms: Number(e.target.value) })}
              className="rounded bg-slate-800 border border-studio-border text-xs px-2 py-1 w-20" aria-label="Fade duration ms" />
          )}
        </div>
        <BusCard label="PROGRAM" name={program?.name} count={program?.sources.length ?? 0} accent="border-red-500" live />
      </div>
    </section>
  );
}

function BusCard({ label, name, count, accent, live }: { label: string; name?: string; count: number; accent: string; live?: boolean }) {
  return (
    <div className={`rounded border-2 ${accent} bg-black/40 p-3 min-h-[120px] flex flex-col`}>
      <div className="flex justify-between items-center">
        <span className="text-[11px] font-bold tracking-widest text-slate-400">{label}</span>
        {live && <span className="text-[10px] font-bold text-red-500 animate-pulse">● LIVE</span>}
      </div>
      <p className="mt-2 text-sm font-semibold truncate">{name ?? '— no scene —'}</p>
      <p className="text-[11px] text-slate-500 mt-auto">{count} source{count === 1 ? '' : 's'}</p>
    </div>
  );
}
