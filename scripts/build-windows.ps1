#Requires -Version 7.0
<#
.SYNOPSIS
  Build di release Windows: controlli statici + test + bundle Tauri.
  Ordine: fmt check -> clippy -D warnings -> cargo test -> typecheck/lint/test frontend -> tauri build.
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

Write-Host '== [1/5] cargo fmt --check ==' -ForegroundColor Cyan
cargo fmt --all -- --check

Write-Host '== [2/5] cargo clippy -D warnings ==' -ForegroundColor Cyan
cargo clippy --workspace --all-targets -- -D warnings

Write-Host '== [3/5] cargo test ==' -ForegroundColor Cyan
cargo test --workspace

Write-Host '== [4/5] frontend: typecheck + lint + test ==' -ForegroundColor Cyan
npm run typecheck
npm run lint
npm run test

Write-Host '== [5/5] tauri build ==' -ForegroundColor Cyan
cargo tauri build

Write-Host "`nBuild completata. Bundle in src-tauri/target/release/bundle." -ForegroundColor Green
