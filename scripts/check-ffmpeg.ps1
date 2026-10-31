#Requires -Version 7.0
$ErrorActionPreference = 'Stop'
Write-Host '=== MY STREAM HUB — verifica FFmpeg ==='
ffmpeg -version 2>&1 | Select-Object -First 1
ffprobe -version 2>&1 | Select-Object -First 1
$enc = & ffmpeg -hide_banner -encoders 2>&1 | Out-String
foreach ($e in @('h264_nvenc', 'h264_amf', 'h264_qsv', 'libx264', 'aac')) {
  if ($enc -match [regex]::Escape($e)) { Write-Host "[OK] $e" } else { Write-Host "[--] $e" }
}
