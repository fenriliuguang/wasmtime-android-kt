# Closed leftover-descriptor-semantics queue. Redirects to webgpu-guest-dawn-remaining.ps1.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"

Write-Host "Playbook closed: docs/agent/webgpu-guest-semantics.md"
Write-Host "Use: .\scripts\webgpu-guest-dawn-remaining.ps1"
Write-Host "Playbook: docs/agent/webgpu-guest-semantics.md"
Write-Host "Next: (F1–F9 empty)"

if (-not $All) { exit 0 }
Write-Host ""
Write-Host "=== (none) ==="
