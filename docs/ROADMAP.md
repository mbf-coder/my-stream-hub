# ROADMAP

- [x] **Phase 1 — Core Backend & Media Runtime**: stati, probe FFmpeg/encoder,
  process manager, 8 comandi IPC, UI diagnostica.
- [x] **Phase 2 — Studio UI/UX**: preview/program, TAKE cut/fade, 32 scene,
  sorgenti (display/window/camera/image/text/media), 12 comandi rundown.
- [x] **Phase 3 — Source Editor & Compositing**: canvas 720p/1080p/4K,
  transform (pos/scale/crop/rotate), builder `filter_complex` validato e testato.
- [x] **Phase 4 — Audio Engine**: mixer 3 canali + master, EQ 3 bande, voice FX
  (pitch/robot/echo), modello VST3 onesto (scan dichiarato non implementato),
  builder `-af` reale da `Mixer`.
- [x] **Phase 5 — Streaming**: destinazioni multi (Twitch/YouTube/Kick/custom),
  OAuth via keyring, key solo in Credential Manager, monitor bitrate/dropped,
  start/stop reali verso RTMP(S).
- [x] **Phase 6 — Recording & Virtual Camera**: MKV/MP4 (+faststart), remux
  stream-copy, registrazione reale, VC con probe onesto (start rifiutato senza driver).
- [x] **Phase 7 — Settings**: `AppConfig` versionato completo, editor UI,
  save/load JSON validato su disco (segreti esclusi).
- [x] **Phase 8 — Hardening & Release**: `stress-test.ps1` (cicli spawn/stop +
  orphan sweep), `RELEASE.md`, CI a due job (Linux core + Windows full).

Regola di conservazione: dopo l'approvazione di una fase, interfacce e
funzionalità sono stabilizzate; modifiche solo per sicurezza/compatibilità/
regressioni, documentate e testate.
