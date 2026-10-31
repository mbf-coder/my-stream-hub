import { Activity } from 'lucide-react';
import { useEngineStore } from '../stores/engineStore';

const STATE_COLOR: Record<string, string> = {
  UNINITIALIZED: 'bg-slate-500',
  INITIALIZING: 'bg-amber-400',
  READY: 'bg-emerald-400',
  RUNNING: 'bg-red-500',
  STOPPING: 'bg-amber-400',
  STOPPED: 'bg-slate-400',
  ERROR: 'bg-red-600',
};

export default function StatusCard() {
  const { status, busy, initialize, refreshStatus } = useEngineStore();
  const state = status?.state ?? 'UNINITIALIZED';
  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4" aria-label="Engine status">
      <div className="flex items-center gap-2 mb-3">
        <Activity size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">ENGINE STATUS</h2>
      </div>
      <div className="flex items-center gap-2 mb-3">
        <span className={`inline-block h-2.5 w-2.5 rounded-full ${STATE_COLOR[state]}`} />
        <span className="font-mono text-lg font-bold">{state}</span>
      </div>
      <dl className="text-xs space-y-1 text-slate-400 mb-4">
        <div className="flex justify-between"><dt>FFmpeg</dt><dd className="text-slate-200 font-mono">{status?.ffmpeg_version ?? '—'}</dd></div>
        <div className="flex justify-between"><dt>FFprobe</dt><dd className="text-slate-200 font-mono">{status?.ffprobe_version ?? '—'}</dd></div>
        <div className="flex justify-between"><dt>Active encoder</dt><dd className="text-slate-200 font-mono">{status?.active_encoder ?? '—'}</dd></div>
        {status?.last_error && (
          <div><dt className="text-red-400">Last error</dt><dd className="text-red-300 break-words">{status.last_error}</dd></div>
        )}
      </dl>
      <div className="flex gap-2">
        <button
          onClick={() => void initialize()}
          disabled={busy}
          className="flex-1 rounded bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 px-3 py-1.5 text-sm font-medium"
        >
          Initialize engine
        </button>
        <button
          onClick={() => void refreshStatus()}
          disabled={busy}
          className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-3 py-1.5 text-sm"
        >
          Refresh
        </button>
      </div>
    </section>
  );
}
