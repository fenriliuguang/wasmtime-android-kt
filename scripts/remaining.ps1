$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$args2 = @()
if ($args -contains "-All" -or $args -contains "--all") { $args2 += "--all" }
& python3 (Join-Path $here "remaining.py") @args2
exit $LASTEXITCODE
