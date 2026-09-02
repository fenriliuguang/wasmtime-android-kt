$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
& python3 (Join-Path $here "product-010-remaining.py") @args
exit $LASTEXITCODE
