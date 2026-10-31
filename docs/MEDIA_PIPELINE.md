# MEDIA PIPELINE (Phase 1 → visione)

## Phase 1: ciclo di vita verificato

```text
spawn(ffmpeg testsrc+sine → libx264/aac → null) → drain stderr/stdout
   → parse progress → engine-stats → stop("q" → wait 5s → kill) → no orphans
```

Ogni figlio nasce con `kill_on_drop(true)`: se il processo Rust muore, l'OS
reclama il figlio. Lo stdin resta aperto per l'invio di `q` (quit grazioso).

## Threading (obiettivo §3.3, progressivo)

| Task | Stato Phase 1 | Prossime fasi |
|---|---|---|
| capture | via FFmpeg dshow adapter | DXGI/WGC nativi |
| audio/video processing | FFmpeg interno al figlio | thread DSP dedicati |
| encoding | `libx264 veryfast` (test) | NVENC/AMF/QSV auto |
| streaming/output | RTMP validato, non trasmesso | Phase 5 |
| monitoring | parser `frame/fps/bitrate/time` | CPU/GPU, dropped frames |

Regole: mai bloccare Tokio con lavoro CPU-bound (`spawn_blocking` quando
servirà), timestamp e A/V sync dal Phase 3 (compositing), ring buffer bounded
(`STDERR_RING_CAP = 200`) per sessioni lunghe senza crescita di memoria.

## Encoder: selezione

Ordine `h264_nvenc > h264_amf > h264_qsv > libx264`, rilevato da
`ffmpeg -encoders` (testuale, nessun SDK toccato). Nulla è presunto: se
l'hardware manca, il software fallback è esplicito in UI (`SW` vs `HW`, `AUTO`).
