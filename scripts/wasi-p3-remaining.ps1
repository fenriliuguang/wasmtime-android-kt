# Closed WASI 0.3 (P1) queue. Redirects to wasmtime-p2-remaining.py.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "wasmtime-p2-remaining.py"
$flags = @()
if ($All) { $flags += "--all" }
Write-Host "Playbook closed: docs/archive/p1-wasi-p3-playbook.md"
Write-Host "Use: python ./scripts/wasmtime-p2-remaining.py"
python $py @flags
