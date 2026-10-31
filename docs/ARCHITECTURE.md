# ARCHITECTURE — MY STREAM HUB (Phase 1)

> Stato: Phase 1 approvata da implementare — Core Backend & Media Runtime.
> Piattaforma di riferimento: **Windows 11 x64**.

## 1. Principi

- **Separazione rigorosa**: UI (React) → IPC tipizzato → orchestrazione Rust →
  processi FFmpeg. Il frontend non esegue mai comandi di sistema.
- **Rust unico orchestratore** dei processi multimediali (`ProcessManager`).
- **FFmpeg come adapter esterno** (processo figlio), non FFI: `kill_on_drop(true)`,
  pipe drenate, timeout, cancellazione cooperativa. FFI solo se un requisito
  futuro lo impone dimostrabilmente.
- **Backpressure e code bounded** tra sottosistemi (capture/audio/video/encode/
  output/monitor) — dettagli in `MEDIA_PIPELINE.md`.
- **Fail-closed sulla sicurezza**: validazione IPC, keyring OS, redazione log.

## 2. Mappa dei moduli Rust (`src-tauri/src`)

| Modulo | Ruolo |
|---|---|
| `state` | Macchina a stati `UNINITIALIZED→INITIALIZING→READY→RUNNING→STOPPING→STOPPED`, più `ERROR` recuperabile. Transizioni validate. |
| `media/engine` | `MediaEngine`: init (probe), start/stop media test, stato condiviso. |
| `media/ffmpeg` | Probe `ffmpeg/ffprobe -version`, parsing versioni, capacità da `-encoders`. |
| `media/encoders` | Rilevamento NVENC→AMF→QSV→libx264 con best-pick automatico. |
| `media/process_manager` | `FfmpegJob` (argv tipizzato) + `ManagedProcess` (stop grazioso → kill, ring stderr bounded). |
| `devices` | Parsing output DirectShow (`dshow`); probe reale solo su Windows. |
| `streaming` | `StreamConfig` validata + builder argv FLV; chiave mai contenuta nel config. |
| `recording` | **Stub Phase 6**: ritorna errore esplicito "non disponibile". |
| `security` | Allow-list schemi RTMP, redazione URL, `validate_path_inside_base`, keyring OS. |
| `configuration` | `AppConfig` JSON (senza segreti) + validazione. |
| `monitoring` | Parser linee di progresso FFmpeg → payload `engine-stats`. |
| `events` | Nomi eventi stabili (`engine-state/error/stats/progress`) + payload. |
| `commands` | Gli 8 comandi IPC Phase 1; `#[tauri::command]` solo in build app. |

## 3. Feature flags Cargo

- `default = ["tauri-app"]`: shell Tauri 2 + keyring reale (build Windows).
- `--no-default-features`: core puro, verificabile su Linux CI senza WebKit/DBus.
  `security::store/load_stream_key` ritornano `CredentialStore` in questa
  configurazione (documentato, non silenzioso).

## 4. Contratto IPC (stabile dopo approvazione Phase 1)

Comandi: `get_engine_status`, `initialize_engine`, `get_media_capabilities`,
`get_available_encoders`, `get_audio_devices`, `get_video_devices`,
`start_media_test`, `stop_media_test`.
Eventi: `engine-state`, `engine-error`, `engine-stats`, `engine-progress`.
Il frontend rispecchia il contratto in `src/types/engine.ts` (+ test di contratto
Rust `tests/ipc_contract.rs` e TS `src/types/engine.test.ts`).

## 5. Frontend (`src/`)

Zustand store (`engineStore`) → `services/tauri.ts` (invoke con fallback browser
tipizzato) → componenti diagnostici. Validazione streaming duplicata in Zod
(`services/stream.ts`) con le stesse regole Rust (test allineati).

## 6. Cosa NON è ancora implementato (onesto per §7 UX)

Scene, sorgenti, compositing, mixer audio, streaming reale, recording, virtual
camera, settings avanzate. La UI Phase 1 mostra solo diagnostica del motore:
nessun pulsante simula funzioni future.
