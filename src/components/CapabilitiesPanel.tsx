import { Cpu, Gauge } from 'lucide-react';
import { useEngineStore } from '../stores/engineStore';

function Check({ ok, label }: { ok: boolean; label: string }) {
  return (
    <li className="flex justify-between text-xs">
      <span className="text-slate-400">{label}</span>
      <span className={ok ? 'text-emerald-400' : 'text-slate-500'}>{ok ? 'YES' : 'NO'}</span>
    </li>
  );
}

export function CapabilitiesPanel() {
  const { capabilities } = useEngineStore();
  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4" aria-label="Media capabilities">
      <div className="flex items-center gap-2 mb-3">
        <Gauge size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">MEDIA CAPABILITIES</h2>
      </div>
      {capabilities ? (
        <ul className="space-y-1">
          <Check ok={capabilities.ffmpeg_version !== null} label={`FFmpeg ${capabilities.ffmpeg_version ?? ''}`} />
          <Check ok={capabilities.ffprobe_version !== null} label={`FFprobe ${capabilities.ffprobe_version ?? ''}`} />
          <Check ok={capabilities.has_libx264} label="libx264 (software fallback)" />
          <Check ok={capabilities.has_aac} label="AAC audio" />
          <Check ok={capabilities.rtmp_supported} label="RTMP output" />
        </ul>
      ) : (
        <p className="text-xs text-slate-500">Not probed yet — initialize the engine.</p>
      )}
    </section>
  );
}

export function EncodersPanel() {
  const { encoders, status } = useEngineStore();
  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4" aria-label="Available encoders">
      <div className="flex items-center gap-2 mb-3">
        <Cpu size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">ENCODERS</h2>
      </div>
      {encoders.length === 0 ? (
        <p className="text-xs text-slate-500">No encoders detected yet.</p>
      ) : (
        <ul className="space-y-1">
          {encoders.map((e) => (
            <li key={e.name} className="flex justify-between text-xs font-mono">
              <span className="text-slate-200">{e.name}</span>
              <span className="flex gap-2 items-center">
                {status?.active_encoder === e.name && (
                  <span className="text-emerald-400 text-[10px] font-sans font-bold">AUTO</span>
                )}
                <span className={e.hardware ? 'text-amber-300' : 'text-slate-400'}>
                  {e.hardware ? 'HW' : 'SW'}
                </span>
              </span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
