import { FlaskConical } from 'lucide-react';
import { useEngineStore } from '../stores/engineStore';

export default function MediaTestPanel() {
  const { status, startTest, stopTest, busy } = useEngineStore();
  const running = status?.state === 'RUNNING';
  const canStart = status?.state === 'READY' || status?.state === 'STOPPED';
  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4" aria-label="Media pipeline test">
      <div className="flex items-center gap-2 mb-3">
        <FlaskConical size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">MEDIA TEST</h2>
      </div>
      <p className="text-xs text-slate-400 mb-3">
        Spawns <code className="font-mono text-slate-200">ffmpeg testsrc → null muxer</code>: proves spawn →
        monitor → stop with no orphan process. No network, no disk writes.
      </p>
      <div className="flex gap-2">
        <button
          onClick={() => void startTest()}
          disabled={busy || !canStart}
          className="flex-1 rounded bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 px-3 py-1.5 text-sm font-medium"
        >
          Start media test
        </button>
        <button
          onClick={() => void stopTest()}
          disabled={busy || !running}
          className="flex-1 rounded bg-red-600 hover:bg-red-500 disabled:opacity-50 px-3 py-1.5 text-sm font-medium"
        >
          Stop media test
        </button>
      </div>
    </section>
  );
}
