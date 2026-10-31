import { useState } from 'react';
import { Radio, CircleDot, Video } from 'lucide-react';
import { useLiveStore } from '../stores/liveStore';
import type { Platform } from '../types/engine';

const PLATFORMS: Platform[] = ['twitch', 'youTube', 'kick', 'custom'];

/** Phase-5 destinations + stream control, Phase-6 recording + virtual camera. */
export default function StreamRecordPanel() {
  const l = useLiveStore();
  const [label, setLabel] = useState('');
  const [url, setUrl] = useState('rtmp://127.0.0.1/live');
  const [platform, setPlatform] = useState<Platform>('custom');
  const [key, setKey] = useState('');
  const [keySaved, setKeySaved] = useState(false);
  const [fileName, setFileName] = useState('show-01.mkv');
  const [remuxIn, setRemuxIn] = useState('show-01.mkv');
  const [remuxOut, setRemuxOut] = useState('show-01.mp4');
  const [remuxDir, setRemuxDir] = useState('recordings');

  return (
    <section className="rounded-lg bg-studio-panel border border-studio-border p-4 md:col-span-2" aria-label="Stream and record">
      <div className="flex items-center gap-2 mb-3">
        <Radio size={16} className="text-slate-400" />
        <h2 className="text-sm font-semibold tracking-wide">STREAM &amp; RECORD</h2>
        {l.stream?.live && <span className="text-[10px] font-bold text-red-500 animate-pulse">● ON AIR{l.stream.destination_label ? ` · ${l.stream.destination_label}` : ''}</span>}
        {l.recording?.recording && <span className="text-[10px] font-bold text-red-400">● REC</span>}
      </div>

      <div className="grid md:grid-cols-2 gap-4">
        <div>
          <p className="text-[11px] text-slate-400 mb-1">Destinations</p>
          <ul className="space-y-1 mb-2 max-h-28 overflow-y-auto">
            {l.destinations.map((d) => (
              <li key={d.id} className="flex items-center gap-1 text-xs bg-slate-800/60 rounded px-2 py-1">
                <span className="flex-1 truncate">{d.label} <span className="text-slate-500">· {d.platform}</span></span>
                <button onClick={() => void l.toggleDestination(d.id)} disabled={l.busy} className={`rounded px-1.5 py-0.5 text-[10px] ${d.enabled ? 'bg-emerald-800' : 'bg-slate-700'}`}>{d.enabled ? 'ON' : 'OFF'}</button>
                <button onClick={() => void l.removeDestination(d.id)} disabled={l.busy} className="text-slate-500 hover:text-red-400 text-[10px]">✕</button>
              </li>
            ))}
            {l.destinations.length === 0 && <p className="text-xs text-slate-500">No destinations.</p>}
          </ul>
          <div className="flex gap-1">
            <select value={platform} onChange={(e) => setPlatform(e.target.value as Platform)} className="rounded bg-slate-800 border border-studio-border text-xs px-1 py-1">
              {PLATFORMS.map((p) => <option key={p} value={p}>{p}</option>)}
            </select>
            <input value={label} onChange={(e) => setLabel(e.target.value)} placeholder="Label" className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1" />
            <input value={url} onChange={(e) => setUrl(e.target.value)} placeholder="rtmp(s)://…" className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1" />
            <button onClick={() => { void l.addDestination(platform, label, url); setLabel(''); }} disabled={l.busy || !label.trim()} className="rounded bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 px-2 py-1 text-xs">Add</button>
          </div>
          <div className="flex gap-1 mt-2">
            <input type="password" value={key} onChange={(e) => setKey(e.target.value)} placeholder="Stream key (keyring only)" autoComplete="off" className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1" />
            <button
              onClick={() => void l.saveStreamKey(key).then((ok) => { if (ok) { setKeySaved(true); setKey(''); } })}
              disabled={l.busy || !key}
              className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-2 py-1 text-xs"
            >
              {keySaved ? 'Saved ✓' : 'Save key'}
            </button>
          </div>
          <div className="flex gap-2 mt-2">
            <button
              onClick={() => {
                const first = l.destinations.find((d) => d.enabled);
                if (first) void l.startStream(first.id, true);
              }}
              disabled={l.busy || l.stream?.live || !l.destinations.some((d) => d.enabled)}
              className="flex-1 rounded bg-red-600 hover:bg-red-500 disabled:opacity-50 px-3 py-1.5 text-xs font-bold"
            >
              GO LIVE (test pattern)
            </button>
            <button onClick={() => void l.stopStream()} disabled={l.busy || !l.stream?.live} className="flex-1 rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-3 py-1.5 text-xs">
              Stop stream
            </button>
          </div>
          <p className="text-[11px] text-slate-500 mt-1">
            {l.stream ? `${l.stream.bitrate_kbps.toFixed(0)} kbps · ${l.stream.dropped_frames} dropped · ${l.stream.uptime_secs}s` : 'Stream idle.'}
          </p>
        </div>

        <div>
          <div className="flex items-center gap-1 mb-1">
            <CircleDot size={12} className="text-slate-400" />
            <p className="text-[11px] text-slate-400">Local recording</p>
          </div>
          <div className="flex gap-1 mb-2">
            <input value={fileName} onChange={(e) => setFileName(e.target.value)} placeholder="show-01.mkv" className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1 font-mono" />
            <button
              onClick={() => void l.startRecording({ file_name: fileName, format: fileName.endsWith('.mp4') ? 'mp4' : 'mkv', tracks: 'mixed', encoder: 'libx264', video_bitrate_kbps: 8000 }, 'recordings')}
              disabled={l.busy || l.recording?.recording || !fileName.trim()}
              className="rounded bg-red-600 hover:bg-red-500 disabled:opacity-50 px-2 py-1 text-xs font-bold"
            >
              ● REC
            </button>
            <button onClick={() => void l.stopRecording()} disabled={l.busy || !l.recording?.recording} className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-2 py-1 text-xs">Stop</button>
          </div>
          <p className="text-[11px] text-slate-500 mb-1 font-mono break-all">{l.recording?.path ?? 'Not recording.'}</p>
          <div className="flex gap-1 mb-1">
            <input value={remuxIn} onChange={(e) => setRemuxIn(e.target.value)} placeholder="in.mkv" className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1 font-mono" />
            <input value={remuxOut} onChange={(e) => setRemuxOut(e.target.value)} placeholder="out.mp4" className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1 font-mono" />
            <button onClick={() => void l.remux(remuxIn, remuxOut, remuxDir)} disabled={l.busy || !remuxIn.trim() || !remuxOut.trim()} className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-2 py-1 text-xs">Remux</button>
          </div>
          <div className="flex gap-1 mb-3 items-center">
            <input value={remuxDir} onChange={(e) => setRemuxDir(e.target.value)} placeholder="recordings" className="flex-1 min-w-0 rounded bg-slate-800 border border-studio-border text-xs px-2 py-1 font-mono" />
            {l.remuxResult && <p className="text-[11px] text-emerald-300 font-mono break-all">→ {l.remuxResult}</p>}
          </div>
          <div className="flex items-center gap-1 mb-1">
            <Video size={12} className="text-slate-400" />
            <p className="text-[11px] text-slate-400">Virtual camera</p>
          </div>
          <div className="flex gap-1">
            <button onClick={() => void l.refreshVc()} disabled={l.busy} className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-2 py-1 text-xs">
              Probe driver
            </button>
            <button onClick={() => void l.startVc()} disabled={l.busy} className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-2 py-1 text-xs">
              Start VC
            </button>
            <button onClick={() => void l.stopVc()} disabled={l.busy} className="rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 px-2 py-1 text-xs">
              Stop VC
            </button>
          </div>
          <p className="text-[11px] text-slate-500 mt-1">
            {l.vc ? (l.vc.driver_installed ? 'Driver found.' : 'No virtual-camera driver — install OBS VirtualCam / Elgato.') : 'Not probed yet.'}
          </p>
        </div>
      </div>
      {l.error && <p className="text-[11px] text-red-300 mt-2 break-words">{l.error}</p>}
    </section>
  );
}
