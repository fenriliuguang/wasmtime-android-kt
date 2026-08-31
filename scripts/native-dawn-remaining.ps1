# Next native-dawn host commit on cursor/native-dawn-rewrite-1355.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "native-dawn-remaining.py"
$flags = @()
if ($All) { $flags += "--all" }
python $py @flags
