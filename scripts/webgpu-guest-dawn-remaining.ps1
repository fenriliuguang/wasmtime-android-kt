# Closed Dawn-consume / WG-6 queue. Redirects to product-010-remaining.ps1.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
Write-Host "Playbook closed: docs/archive/webgpu-guest-dawn.md"
Write-Host "Use: .\scripts\product-010-remaining.ps1"
& (Join-Path $PSScriptRoot "product-010-remaining.ps1") @PSBoundParameters
