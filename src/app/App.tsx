import { useEffect, useState } from 'react';
import StatusCard from '../components/StatusCard';
import { CapabilitiesPanel, EncodersPanel } from '../components/CapabilitiesPanel';
import DevicesPanel from '../components/DevicesPanel';
import MediaTestPanel from '../components/MediaTestPanel';
import LogPanel from '../components/LogPanel';
import StudioView from '../components/StudioView';
import ScenesSourcesPanel from '../components/ScenesSourcesPanel';
import MixerPanel from '../components/MixerPanel';
import StreamRecordPanel from '../components/StreamRecordPanel';
import SettingsPanel from '../components/SettingsPanel';
import { useEngineStore } from '../stores/engineStore';
import { useStudioStore } from '../stores/studioStore';
import { useLiveStore } from '../stores/liveStore';
import { useSettingsStore } from '../stores/settingsStore';
import { isTauriRuntime } from '../services/tauri';

type Tab = 'studio' | 'stream' | 'audio' | 'settings' | 'diagnostics';

const TABS: { id: Tab; label: string }[] = [
  { id: 'studio', label: 'Studio' },
  { id: 'stream', label: 'Stream & Record' },
  { id: 'audio', label: 'Audio' },
  { id: 'settings', label: 'Settings' },
  { id: 'diagnostics', label: 'Diagnostics' },
];

/** MY STREAM HUB shell — Phase 2–8 studio UI with Phase-1 diagnostics tab. */
export default function App() {
  const [tab, setTab] = useState<Tab>('studio');
  const refreshStatus = useEngineStore((s) => s.refreshStatus);
  const refreshScenes = useStudioStore((s) => s.refresh);
  const refreshMixer = useLiveStore((s) => s.refreshMixer);
  const refreshDestinations = useLiveStore((s) => s.refreshDestinations);
  const loadSettings = useSettingsStore((s) => s.load);
  useEffect(() => {
    if (!isTauriRuntime()) return;
    refreshStatus().catch(() => undefined);
    refreshScenes().catch(() => undefined);
    refreshMixer().catch(() => undefined);
    refreshDestinations().catch(() => undefined);
    loadSettings('settings').catch(() => undefined);
  }, []);
  return (
    <div className="min-h-screen">
      <header className="border-b border-studio-border px-6 py-3 flex items-center justify-between">
        <div>
          <h1 className="text-base font-bold tracking-widest">MY STREAM HUB</h1>
          <p className="text-[11px] text-slate-500">Professional Native Live Streaming Studio</p>
        </div>
        {!isTauriRuntime() && (
          <p className="text-[11px] text-amber-300 bg-amber-950 border border-amber-800 rounded px-2 py-1">
            Browser preview — IPC requires the Tauri desktop shell
          </p>
        )}
      </header>
      <nav className="px-4 pt-3 flex gap-1 max-w-7xl mx-auto" aria-label="Studio sections">
        {TABS.map((t) => (
          <button key={t.id} onClick={() => setTab(t.id)}
            className={`rounded-t px-4 py-1.5 text-xs font-semibold tracking-wide ${tab === t.id ? 'bg-studio-panel text-white border border-b-0 border-studio-border' : 'text-slate-400 hover:text-slate-200'}`}>
            {t.label.toUpperCase()}
          </button>
        ))}
      </nav>
      <main className="p-4 grid gap-4 md:grid-cols-2 xl:grid-cols-3 max-w-7xl mx-auto">
        {tab === 'studio' && (<><StudioView /><ScenesSourcesPanel /><LogPanel /></>)}
        {tab === 'stream' && (<><StreamRecordPanel /><LogPanel /></>)}
        {tab === 'audio' && (<><MixerPanel /><LogPanel /></>)}
        {tab === 'settings' && (<><SettingsPanel /><DevicesPanel /></>)}
        {tab === 'diagnostics' && (<><StatusCard /><CapabilitiesPanel /><EncodersPanel /><DevicesPanel /><MediaTestPanel /><LogPanel /></>)}
      </main>
    </div>
  );
}
