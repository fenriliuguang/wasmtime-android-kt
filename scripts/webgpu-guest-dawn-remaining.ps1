# Closed Dawn-consume / WG-6 queue. Redirects to wasi-p3-remaining.ps1.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"

Write-Host "Playbook closed: docs/archive/webgpu-guest-dawn.md"
Write-Host "Use: .\scripts\wasi-p3-remaining.ps1"
Write-Host "Playbook: docs/agent/wasi-p3.md"
Write-Host "Next: (G1–G9 empty; P0 closed)"

if (-not $All) { exit 0 }
Write-Host ""
Write-Host "=== (none) ==="
