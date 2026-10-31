import { Mic, Video } from 'lucide-react';
import { useEngineStore } from '../stores/engineStore';

export default function DevicesPanel() {
  const { audioDevices, videoDevices, loadDevices, busy } = useEngineStore();
  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4" aria-label="Capture devices">
      <div className="flex items-center justify-between mb-3">
        <h2 className="text-sm font-semibold tracking-wide">DEVICES</h2>
        <button
          onClick={() => void loadDevices()}
          disabled={busy}
          className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-2 py-1 text-xs"
        >
          Probe devices
        </button>
      </div>
      <div className="grid grid-cols-2 gap-3 text-xs">
        <div>
          <div className="flex items-center gap-1 text-slate-400 mb-1"><Video size={12} /><span>Video</span></div>
          {videoDevices.length === 0
            ? <p className="text-slate-500">—</p>
            : videoDevices.map((d) => <p key={d.name} className="font-mono text-slate-200 truncate" title={d.detail}>{d.name}</p>)}
        </div>
        <div>
          <div className="flex items-center gap-1 text-slate-400 mb-1"><Mic size={12} /><span>Audio</span></div>
          {audioDevices.length === 0
            ? <p className="text-slate-500">—</p>
            : audioDevices.map((d) => <p key={d.name} className="font-mono text-slate-200 truncate" title={d.detail}>{d.name}</p>)}
        </div>
      </div>
      <p className="text-[11px] text-slate-500 mt-2">Device probing requires Windows 11 (DirectShow).</p>
    </section>
  );
}
