#Requires -Version 7.0
<# .SYNOPSIS Rimuove gli artefatti di build (target, dist, node_modules). #>
$ErrorActionPreference = 'Continue'
$root = Split-Path $PSScriptRoot -Parent
foreach ($d in @("$root\src-tauri\target", "$root\dist", "$root\node_modules")) {
  if (Test-Path $d) { Remove-Item $d -Recurse -Force; Write-Host "Rimosso: $d" }
}
Write-Host 'Pulizia completata.'
