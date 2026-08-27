# Closed WASI 0.3 (P1) queue. Redirects to product-010-remaining.py.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "product-010-remaining.py"
$flags = @()
if ($All) { $flags += "--all" }
Write-Host "Playbook closed: docs/archive/p1-wasi-p3-playbook.md"
Write-Host "Use: python ./scripts/product-010-remaining.py"
python $py @flags
