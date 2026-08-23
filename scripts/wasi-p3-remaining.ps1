# Next WASI 0.3 (P1) PR. Agents: run this instead of grepping cm.rs.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "wasi-p3-remaining.py"
$flags = @()
if ($All) { $flags += "--all" }
python $py @flags
