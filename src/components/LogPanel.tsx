import { Terminal } from 'lucide-react';
import { useEngineStore } from '../stores/engineStore';

export default function LogPanel() {
  const { log, error } = useEngineStore();
  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4" aria-label="Diagnostics log">
      <div className="flex items-center gap-2 mb-3">
        <Terminal size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">DIAGNOSTICS LOG</h2>
      </div>
      {error && <p className="text-xs text-red-300 mb-2 break-words">{error}</p>}
      <div className="h-40 overflow-y-auto font-mono text-[11px] text-slate-300 space-y-0.5" role="log">
        {log.length === 0
          ? <p className="text-slate-500">No events yet.</p>
          : log.map((line, i) => <p key={i}>{line}</p>)}
      </div>
    </section>
  );
}
