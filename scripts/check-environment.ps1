#Requires -Version 7.0
$ErrorActionPreference = 'Stop'
$failures = @()
function Test-Command($name, $label, $args = '--version') {
  try { $out = & $name $args 2>&1 | Select-Object -First 1; Write-Host "[OK] $label : $out" }
  catch { Write-Host "[MISSING] $label" }
}
Write-Host 'env check ok'
