# Android Dawn C API .so recipe (ND-SO). Pin: native/third_party/dawn-c/ORIGIN.txt
param(
    [switch]$ProbeAar,
    [switch]$Build,
    [switch]$Prebuilt,
    [string[]]$Targets = @("arm64-v8a")
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
$py = Join-Path $here "build-dawn-c-android.py"
$flags = @()
if ($ProbeAar) { $flags += "--probe-aar" }
if ($Prebuilt) { $flags += "--prebuilt" }
if ($Build) { $flags += "--build" }
if (($Prebuilt -or $Build) -and $Targets.Count -gt 0) {
    $flags += "--targets"
    $flags += $Targets
}
if (-not $ProbeAar -and -not $Build -and -not $Prebuilt) {
    $flags = @("--probe-aar")
}
python $py @flags
