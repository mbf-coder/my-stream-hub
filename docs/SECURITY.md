# SECURITY (Phase 1)

## Regole inviolabili

1. **Stream key mai in chiaro**: vive solo in Windows Credential Manager
   (`keyring`, service `com.mystreamhub.studio`, account `active-stream-key`).
   Mai in config JSON, log, errori IPC o payload eventi.
2. **Redazione**: i diagnostici usano `redacted_rtmp_url`
   (`rtmp://host/app/***`). Il builder `output_args` riceve l'ingest completo
   solo al momento dello spawn, dal keyring — mai dallo store UI.
3. **Allow-list URL**: solo `rtmp://` e `rtmps://`, max 2048 char, nessuno
   spazio. Niente shell: gli argv FFmpeg sono array, mai stringhe interpretate.
4. **Path traversal**: `validate_path_inside_base` normalizza lessicalmente e
   rifiuta `..` che escono dalla base (recording/stinger, Phase 3/6).
5. **Tauri least-privilege**: `capabilities/default.json` senza permessi extra;
   superficie IPC = 8 comandi tipizzati con parametri validati.
6. **Nessun servizio di rete** esposto in Phase 1 (nessuna porta in ascolto).

## Verifiche

- Test `security::tests_*`: schemi pericolosi, redazione, escape, key vuote.
- Test `streaming::tests_*`: config valida/rifiutata, muxer FLV, nessuna chiave nei log.
- Audit manuale: `grep -ri "live_" src-tauri/src` non deve mostrare segreti.
