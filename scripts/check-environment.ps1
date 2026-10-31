#Requires -Version 7.0
$ErrorActionPreference = 'Stop'
$failures = @()
function Test-Command($name, $label, $args = '--version') {
  try { $out = & $name $args 2>&1 | Select-Object -First 1; Write-Host "[OK] $label : $out" }
  catch { Write-Host "[MISSING] $label"; $script:failures += $label }
}
Write-Host '=== MY STREAM HUB — controllo ambiente ==='
Test-Command 'rustc' 'Rust toolchain'
Test-Command 'cargo' 'Cargo'
Test-Command 'node' 'Node.js LTS'
Test-Command 'npm' 'npm'
Test-Command 'git' 'Git'
Test-Command 'ffmpeg' 'FFmpeg' '-version'
Test-Command 'ffprobe' 'FFprobe' '-version'
if ($failures.Count -gt 0) { exit 1 }
Write-Host 'Ambiente pronto.'
