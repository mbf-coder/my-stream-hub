# RELEASE — Phase 8 Hardening & Checklist

## Pre-release (ogni versione)

1. `.\scripts\check-environment.ps1` verde su macchina Windows 11 pulita.
2. `.\scripts\build-windows.ps1` verde (fmt → clippy `-D warnings` → Rust test →
   typecheck/lint/test frontend → `cargo tauri build`).
3. `.\scripts\stress-test.ps1 -Cycles 50` senza orfani.
4. Smoke manuale (30 min live verso destinazione di test + registrazione MKV
   simultanea): nessun crash, nessun frame-drop anomalo, log senza segreti.
5. `grep -ri "live_" src-tauri/src src` non mostra segreti; `oauth` solo in keyring.

## Versionamento

- `package.json` + `src-tauri/Cargo.toml` + `tauri.conf.json`: stessa versione.
- CHANGELOG in testa al README (sezione Release): data, fasi incluse, fix.

## Pacchettizzazione

- Target: `src-tauri/target/release/bundle/` (MSI + NSIS).
- Firma codice: certificato dell'editore; senza firma SmartScreen avverte —
  documentarlo nella pagina di download, non aggirarlo.
- Rollback: conservare MSI della versione precedente + `mystreamhub.config.json`
  di backup prima di ogni upgrade (il formato settings ha `version: 1`).

## Note di onestà

- Virtual camera: lo start rifiuta finché non c'è un driver (OBS VirtualCam /
  Elgato). Non spedire mai uno stato "attivo" falso.
- Encoder HW: riportare solo ciò che `ffmpeg -encoders` rileva davvero.
- VST3: modello + validazione directory; hosting reale in milestone futura.
