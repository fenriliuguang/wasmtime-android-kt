# Closed leftover-descriptor-semantics queue. Redirects to wasmtime-p2-remaining.ps1.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"

Write-Host "Playbook closed: docs/archive/webgpu-guest-semantics.md"
Write-Host "Use: .\scripts\wasmtime-p2-remaining.ps1"
Write-Host "Playbook: docs/agent/wasmtime-p2.md"
Write-Host "Next: (F1–F9 empty; P0 closed)"

if (-not $All) { exit 0 }
Write-Host ""
Write-Host "=== (none) ==="
