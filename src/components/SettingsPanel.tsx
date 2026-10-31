import { useState } from 'react';
import { Settings2 } from 'lucide-react';
import { useSettingsStore } from '../stores/settingsStore';

/** Phase-7 full settings editor (general + video + audio + stream/recording defaults). */
export default function SettingsPanel() {
  const { settings, savedPath, busy, error, patch, save } = useSettingsStore();
  const [baseDir, setBaseDir] = useState('settings');
  if (!settings) return null;
  const num = (v: string, fallback: number) => {
    const n = Number(v);
    return Number.isFinite(n) ? n : fallback;
  };
  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4 md:col-span-2" aria-label="Settings">
      <div className="flex items-center gap-2 mb-3">
        <Settings2 size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">SETTINGS</h2>
      </div>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs mb-3">
        <label className="flex flex-col gap-1">Output W
          <input type="number" value={settings.output_width} disabled={busy} onChange={(e) => patch({ output_width: num(e.target.value, 1920) })} className="rounded bg-slate-800 border border-studio-border px-2 py-1" />
        </label>
        <label className="flex flex-col gap-1">Output H
          <input type="number" value={settings.output_height} disabled={busy} onChange={(e) => patch({ output_height: num(e.target.value, 1080) })} className="rounded bg-slate-800 border border-studio-border px-2 py-1" />
        </label>
        <label className="flex flex-col gap-1">FPS
          <input type="number" value={settings.fps} disabled={busy} onChange={(e) => patch({ fps: num(e.target.value, 60) })} className="rounded bg-slate-800 border border-studio-border px-2 py-1" />
        </label>
        <label className="flex flex-col gap-1">Sample rate
          <select value={settings.sample_rate} disabled={busy} onChange={(e) => patch({ sample_rate: Number(e.target.value) })} className="rounded bg-slate-800 border border-studio-border px-2 py-1">
            <option value={44100}>44100</option>
            <option value={48000}>48000</option>
            <option value={96000}>96000</option>
          </select>
        </label>
        <label className="flex flex-col gap-1">Encoder
          <input value={settings.stream.encoder} disabled={busy} onChange={(e) => patch({ stream: { ...settings.stream, encoder: e.target.value } })} className="rounded bg-slate-800 border border-studio-border px-2 py-1 font-mono" />
        </label>
        <label className="flex flex-col gap-1">Video kbps
          <input type="number" value={settings.stream.video_bitrate_kbps} disabled={busy} onChange={(e) => patch({ stream: { ...settings.stream, video_bitrate_kbps: num(e.target.value, 6000) } })} className="rounded bg-slate-800 border border-studio-border px-2 py-1" />
        </label>
        <label className="flex flex-col gap-1">Audio kbps
          <select value={settings.stream.audio_bitrate_kbps} disabled={busy} onChange={(e) => patch({ stream: { ...settings.stream, audio_bitrate_kbps: Number(e.target.value) })} className="rounded bg-slate-800 border border-studio-border px-2 py-1">
            {[64, 96, 128, 160, 192, 256, 320].map((b) => <option key={b} value={b}>{b}</option>)}
          </select>
        </label>
        <label className="flex flex-col gap-1">Rec format
          <select value={settings.recording.format} disabled={busy} onChange={(e) => patch({ recording: { ...settings.recording, format: e.target.value } })} className="rounded bg-slate-800 border border-studio-border px-2 py-1">
            <option value="mkv">mkv</option>
            <option value="mp4">mp4</option>
          </select>
        </label>
      </div>
      <div className="flex gap-2 items-center">
        <input value={baseDir} onChange={(e) => setBaseDir(e.target.value)} placeholder="settings dir" className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1 font-mono" />
        <button onClick={() => void save(baseDir)} disabled={busy} className="rounded bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 px-3 py-1.5 text-xs font-medium">Save settings</button>
      </div>
      {savedPath && <p className="text-[11px] text-emerald-300 mt-1 font-mono break-all">Saved → {savedPath}</p>}
      {error && <p className="text-[11px] text-red-300 mt-1 break-words">{error}</p>}
    </section>
  );
}