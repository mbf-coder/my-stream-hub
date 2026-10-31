#Requires -Version 7.0
$ErrorActionPreference = 'Stop'
if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
  Write-Host '[FATAL] winget non trovato.'
  exit 1
}
foreach ($id in @('Rustlang.Rustup', 'Microsoft.VisualStudio.2022.BuildTools', 'OpenJS.NodeJS.LTS', 'Microsoft.EdgeWebView2Runtime', 'Gyan.FFmpeg', 'Git.Git', 'Microsoft.PowerShell')) {
  winget install --id $id --silent --accept-package-agreements --accept-source-agreements
}
