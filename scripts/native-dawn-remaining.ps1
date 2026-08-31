# Next native-dawn host PR. Agents: run this instead of grepping cm.rs.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "native-dawn-remaining.py"
$flags = @()
if ($All) { $flags += "--all" }
python $py @flags
