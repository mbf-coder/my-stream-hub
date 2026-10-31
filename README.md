# MY STREAM HUB

> **Professional Native Live Streaming Studio** — Rust · Tauri 2 · React · FFmpeg
> Stato: **Phase 1–8 complete** — Core, Studio, Compositing, Audio, Streaming,
Recording, Settings, Hardening.

Applicazione desktop nativa per produzione e trasmissione live, eseguita
interamente sul computer dell'utente: nessun cloud obbligatorio, nessun
abbonamento intermedio per trasmettere.

## Funzionalità (verificate da test automatici)

**Phase 1 — Core**: backend Rust + shell Tauri 2, `MediaEngine` a stati,
probe FFmpeg/encoder HW, `ProcessManager`, media test `testsrc → null`,
8 comandi IPC, `StreamConfig` validata (Rust+Zod), key solo in Credential Manager.

**Phase 2 — Studio**: bus preview/program + TAKE (cut/fade), 32 scene,
sorgenti display/window/camera/image/text/media, layering e visibilità.

**Phase 3 — Compositing**: canvas 720p/1080p/4K, transform per layer,
builder `filter_complex` FFmpeg validato.

**Phase 4 — Audio**: mixer mic/desktop/aux + master, EQ 3 bande, voice FX,
builder `-af` reale, VST3 dichiarato onestamente come non implementato.

**Phase 5 — Streaming**: destinazioni multi, OAuth in keyring, start/stop RTMP(S)
reali, monitor salute live.

**Phase 6 — Recording & VC**: MKV/MP4 + remux stream-copy, virtual camera con
probe onesto (niente stati falsi).

**Phase 7 — Settings**: configurazione versionata completa, editor UI, save/load JSON.

**Phase 8 — Hardening**: `scripts/stress-test.ps1`, checklist `docs/RELEASE.md`,
CI Linux + Windows.

## Funzionalità storiche Phase 1 (dettaglio)

- [x] Backend Rust + shell Tauri 2 con capability least-privilege
- [x] `MediaEngine` con macchina a stati validata (+ `ERROR` recuperabile)
- [x] Rilevamento FFmpeg/FFprobe (versione) e parsing capacità
- [x] Rilevamento encoder NVENC → AMF → QSV → libx264 con best-pick automatico
- [x] `ProcessManager`: spawn/stop grazioso/kill, pipe drenate, nessun orfano
- [x] Media test `testsrc → null muxer` (nessuna rete, nessuna scrittura disco)
- [x] 8 comandi IPC tipizzati + eventi `engine-state/error/stats/progress`
- [x] Validazione `StreamConfig` (Rust + Zod allineati), redazione stream key
- [x] Stream key solo in Windows Credential Manager (keyring OS)
- [x] UI diagnostica scura: stato, capacità, encoder, dispositivi, test, log
- [x] Script PowerShell per toolchain Windows 11 e build release

## Limiti onesti (non nascosti)

- Virtual camera: start rifiutato senza driver installato (OBS VirtualCam / Elgato).
- VST3: modello + validazione; hosting reale in milestone futura.
- Encoder HW e dispositivi reali: rilevati solo ciò che FFmpeg/`dshow` riportano
  sul PC (verificare su Windows 11 con `./scripts/check-environment.ps1`).
Dettagli in [`docs/RELEASE.md`](docs/RELEASE.md) e [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Requisiti (Windows 11 x64)

Rust stable · Cargo · VS Build Tools (MSVC) · Windows 10/11 SDK · Node.js LTS ·
npm · Tauri CLI 2 · WebView2 Runtime · FFmpeg + FFprobe · Git · PowerShell 7.

```powershell
.\scripts\install-windows.ps1
cargo install tauri-cli --version ^2
.\scripts\check-environment.ps1
```

## Sviluppo / Build / Test

```powershell
.\scripts\run-dev.ps1        # dev: Vite + Tauri
.\scripts\build-windows.ps1  # fmt → clippy → test → frontend → tauri build
```

Dettagli: [`docs/BUILD.md`](docs/BUILD.md) · [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) ·
[`docs/MEDIA_PIPELINE.md`](docs/MEDIA_PIPELINE.md) · [`docs/SECURITY.md`](docs/SECURITY.md).

## Sicurezza

Stream key solo in Credential Manager, mai in log/config/errori. Validazione di
tutti i parametri IPC, allow-list `rtmp(s)`, protezione path traversal, nessun
servizio di rete esposto. Dettagli in [`docs/SECURITY.md`](docs/SECURITY.md).

## Licenza

Apache-2.0 — vedi `LICENSE` nel repository.
