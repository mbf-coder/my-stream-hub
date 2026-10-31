import { useState } from 'react';
import { SlidersHorizontal, VolumeX, Volume2 } from 'lucide-react';
import { useLiveStore } from '../stores/liveStore';
import type { VoiceFx } from '../types/engine';

/** Phase-4 mixer: 3 channels + master, EQ, voice FX, VST3 scan. */
export default function MixerPanel() {
  const { mixer, busy, error, setGain, setMute, setMasterGain, setMasterMute, setEq, setVoiceFx, vstNote, scanVst } =
    useLiveStore();
  const [vstDir, setVstDir] = useState('C:\\Program Files\\Common Files\\VST3');
  if (!mixer) return null;

  const fxValue: string =
    mixer.voice_fx === 'none' || mixer.voice_fx === 'robot' || mixer.voice_fx === 'echo'
      ? mixer.voice_fx
      : 'pitch';

  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4" aria-label="Audio mixer">
      <div className="flex items-center gap-2 mb-3">
        <SlidersHorizontal size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">MIXER</h2>
      </div>
      <div className="grid grid-cols-3 gap-2 mb-3">
        {mixer.channels.map((ch) => (
          <div key={ch.id} className="rounded bg-slate-800/60 p-2 text-center">
            <p className="text-[11px] font-semibold capitalize">{ch.id}</p>
            <input
              type="range"
              min={-60}
              max={12}
              step={1}
              value={ch.gain_db}
              disabled={busy}
              onChange={(e) => void setGain(ch.id, Number(e.target.value))}
              className="w-full"
              aria-label={`${ch.id} gain dB`}
            />
            <p className="text-[11px] font-mono">{ch.gain_db.toFixed(0)} dB</p>
            <button
              onClick={() => void setMute(ch.id, !ch.muted)}
              disabled={busy}
              className={`mt-1 rounded px-2 py-0.5 text-[11px] ${ch.muted ? 'bg-red-700' : 'bg-slate-700 hover:bg-slate-600'}`}
            >
              {ch.muted ? <VolumeX size={12} /> : <Volume2 size={12} />}
            </button>
          </div>
        ))}
      </div>
      <div className="flex items-center gap-2 text-xs mb-3">
        <span className="text-slate-400">Master</span>
        <input
          type="range" min={-60} max={12} step={1}
          value={mixer.master_gain_db} disabled={busy}
          onChange={(e) => void setMasterGain(Number(e.target.value))}
          className="flex-1" aria-label="Master gain dB"
        />
        <span className="font-mono">{mixer.master_gain_db.toFixed(0)} dB</span>
        <button
          onClick={() => void setMasterMute(!mixer.master_muted)}
          disabled={busy}
          className={`rounded px-2 py-0.5 text-[11px] ${mixer.master_muted ? 'bg-red-700' : 'bg-slate-700 hover:bg-slate-600'}`}
        >
          {mixer.master_muted ? 'MUTED' : 'MUTE'}
        </button>
      </div>
      <div className="grid grid-cols-3 gap-2 text-xs mb-3">
        {(['low', 'mid', 'high'] as const).map((band) => (
          <label key={band} className="flex flex-col gap-1">
            <span className="text-slate-400 capitalize">EQ {band} · {mixer.eq[band].freq_hz} Hz</span>
            <input
              type="number" min={-24} max={24} step={0.5}
              value={mixer.eq[band].gain_db} disabled={busy}
              onChange={(e) => {
                const gain_db = Number(e.target.value);
                void setEq({ ...mixer.eq, [band]: { ...mixer.eq[band], gain_db } });
              }}
              className="rounded bg-slate-800 border border-studio-border px-2 py-1 text-xs"
            />
          </label>
        ))}
      </div>
      <label className="flex items-center gap-2 text-xs">
        <span className="text-slate-400">Voice FX</span>
        <select
          value={fxValue}
          disabled={busy}
          onChange={(e) => {
            const v = e.target.value;
            const fx: VoiceFx = v === 'pitch' ? { pitch_shift: -200 } : (v as VoiceFx);
            void setVoiceFx(fx);
          }}
          className="rounded bg-slate-800 border border-studio-border px-2 py-1 text-xs"
        >
          <option value="none">none</option>
          <option value="pitch">pitch −200¢</option>
          <option value="robot">robot</option>
          <option value="echo">echo</option>
        </select>
      </label>
      <div className="flex gap-1 mt-3 items-center">
        <input value={vstDir} onChange={(e) => setVstDir(e.target.value)} placeholder="VST3 dir" className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1 font-mono" />
        <button onClick={() => void scanVst(vstDir)} disabled={busy || !vstDir.trim()} className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-2 py-1 text-xs">Scan VST3</button>
      </div>
      {vstNote && <p className="text-[11px] text-slate-500 mt-1 break-words">{vstNote}</p>}
      {error && <p className="text-[11px] text-red-300 mt-2 break-words">{error}</p>}
    </section>
  );
}
