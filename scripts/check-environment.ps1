#Requires -Version 7.0
$ErrorActionPreference = 'Stop'
$failures = @()
function Test-Command($name, $label, $args = '--version') {
  try { $out = & $name $args 2>&1 | Select-Object -First 1; Write-Host "[OK] $label : $out" }
  catch { Write-Host "[MISSING] $label"; $script:failures += $label }
}
Write-Host '=== MY STREAM HUB — controllo ambiente (Windows 11 x64) ==='
Test-Command 'rustc' 'Rust toolchain'
Test-Command 'cargo' 'Cargo'
Test-Command 'node' 'Node.js LTS'
Test-Command 'npm' 'npm'
Test-Command 'git' 'Git'
Test-Command 'ffmpeg' 'FFmpeg' '-version'
Test-Command 'ffprobe' 'FFprobe' '-version'
$pf86 = [Environment]::GetFolderPath('ProgramFilesX86')
$vswhere = Join-Path $pf86 'Microsoft Visual Studio/Installer/vswhere.exe'
if (Test-Path $vswhere) { Write-Host '[OK] MSVC C++ Build Tools' }
else { Write-Host '[MISSING] MSVC Build Tools'; $failures += 'MSVC' }
$sdkRoot = Join-Path $pf86 'Windows Kits/10/bin'
if (Test-Path $sdkRoot) { Write-Host '[OK] Windows 10/11 SDK' }
else { Write-Host '[MISSING] Windows 10/11 SDK'; $failures += 'SDK' }
try { $t = cargo tauri --version 2>&1 | Select-Object -First 1; Write-Host "[OK] Tauri CLI : $t" }
catch { Write-Host '[MISSING] Tauri CLI'; $failures += 'TauriCLI' }
if ($failures.Count -gt 0) { Write-Host ("Mancano: " + ($failures -join ', ')); exit 1 }
Write-Host 'Ambiente pronto.'
