# Closed guest-pipeline queue. Redirects to webgpu-guest-dawn-remaining.ps1.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"

Write-Host "Playbook closed: docs/agent/webgpu-guest-pipeline.md"
Write-Host "Use: .\scripts\webgpu-guest-dawn-remaining.ps1"
Write-Host "Playbook: docs/agent/webgpu-guest-pipeline.md"
Write-Host "Next: (P1–P5 empty)"

if (-not $All) { exit 0 }
Write-Host ""
Write-Host "=== (none) ==="
