# In-tree press gate: published AARs must contain this checkout's recipe .so.
# Does not clone or assemble wasmtime-android-kt-examples.
param(
    [switch]$Assemble
)

$ErrorActionPreference = "Stop"
$py = Join-Path $PSScriptRoot "verify-press-aar.py"
$flags = @()
if ($Assemble) { $flags += "--assemble" }
python $py @flags
