$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
& python3 (Join-Path $here "remaining.py") @args
exit $LASTEXITCODE
