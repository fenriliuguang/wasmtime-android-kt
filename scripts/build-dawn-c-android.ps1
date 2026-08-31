# Android Dawn C API .so recipe (ND-SO). Pin: native/third_party/dawn-c/ORIGIN.txt
param(
    [switch]$ProbeAar,
    [switch]$Build,
    [string[]]$Targets = @("arm64-v8a")
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "build-dawn-c-android.py"
$flags = @()
if ($ProbeAar) { $flags += "--probe-aar" }
if ($Build) { $flags += "--build" }
if ($Build -and $Targets.Count -gt 0) {
    $flags += "--targets"
    $flags += $Targets
}
if (-not $ProbeAar -and -not $Build) {
    $flags = @("--probe-aar")
}
python $py @flags
