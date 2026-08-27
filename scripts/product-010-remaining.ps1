# Next 0.1.0 product-gate PR. Agents: run this instead of grepping cm.rs.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "product-010-remaining.py"
$flags = @()
if ($All) { $flags += "--all" }
python $py @flags
