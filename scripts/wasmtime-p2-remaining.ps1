# Next Wasmtime pin (P2) PR. Agents: run this instead of grepping Cargo.toml.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "wasmtime-p2-remaining.py"
$flags = @()
if ($All) { $flags += "--all" }
python $py @flags
