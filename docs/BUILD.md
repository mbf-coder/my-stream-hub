# BUILD — Windows 11 x64

## Prerequisiti

```powershell
.\scripts\install-windows.ps1        # winget: Rust, MSVC, SDK, Node LTS, WebView2, FFmpeg, Git, pwsh7
cargo install tauri-cli --version ^2
.\scripts\check-environment.ps1      # deve uscire "Ambiente pronto."
```

## Sviluppo

```powershell
.\scripts\run-dev.ps1                # verifica FFmpeg + npm install + cargo tauri dev
```

## Release

```powershell
.\scripts\build-windows.ps1
```

Esegue in ordine: `cargo fmt --check` → `clippy -D warnings` → `cargo test`
→ `npm run typecheck/lint/test` → `cargo tauri build` (bundle MSI/NSIS in
`src-tauri/target/release/bundle`).

## Verifica senza Windows (CI Linux)

Il core Rust compila e si testa senza WebKit:

```bash
cd src-tauri && cargo fmt --all -- --check
cargo clippy --no-default-features --all-targets -- -D warnings
cargo test --no-default-features
```

Frontend ovunque: `npm run typecheck && npm run lint && npm run test && npm run build`.

## Note

- `cargo tauri build` richiede WebView2 + MSVC: solo Windows.
- La pipeline FFmpeg di test (`testsrc → null`) non richiede rete né disco.
