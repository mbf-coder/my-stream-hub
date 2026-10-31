#Requires -Version 7.0
$ErrorActionPreference = 'Stop'
param([int]$Cycles = 20)
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root
Write-Host '== [1/3] full test suites ==' 
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit 1 }
npm run test --silent
if ($LASTEXITCODE -ne 0) { exit 1 }
Write-Host '== [2/3] spawn/stop cycles ==' 
$leaked = 0
for ($i = 1; $i -le $Cycles; $i++) {
  $p = Start-Process ffmpeg -ArgumentList @(
    '-hide_banner', '-loglevel', 'error',
    '-f', 'lavfi', '-i', 'testsrc=size=1280x720:rate=30',
    '-f', 'lavfi', '-i', 'sine=frequency=1000:sample_rate=48000',
    '-t', '3600', '-c:v', 'libx264', '-preset', 'veryfast',
    '-c:a', 'aac', '-f', 'null', '-'
  ) -PassThru -NoNewWindow
  Start-Sleep -Milliseconds 800
  if ($p.HasExited) { continue }
  Stop-Process -Id $p.Id -Force
  $p.WaitForExit(5000) | Out-Null
  if (-not $p.HasExited) { $leaked++ }
}
Write-Host '== [3/3] orphan sweep ==' 
$orphans = Get-Process ffmpeg -ErrorAction SilentlyContinue
if ($orphans) { exit 1 }
if ($leaked -gt 0) { exit 1 }
Write-Host '[OK] no orphans, suites green.'
