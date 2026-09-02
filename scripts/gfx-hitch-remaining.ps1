# Next cube-hitch restart commit on fix/300-gfx-cube-pop.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "gfx-hitch-remaining.py"
$flags = @()
if ($All) { $flags += "--all" }
python $py @flags
