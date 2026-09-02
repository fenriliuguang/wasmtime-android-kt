# Out-of-tree examples gate: includeBuild this checkout (no mavenLocal).
# Default examples tree: sibling ../wasmtime-android-kt-examples or $env:EXAMPLES_DIR.
# Always :app:assembleDebug. :app:installDebug when an adb device is present
# (or -Install). Cube guest: guests/rotating-cube/dist/guest.wasm.
param(
    [string]$ExamplesDir = $env:EXAMPLES_DIR,
    [switch]$Install,
    [switch]$AssembleOnly
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

if (-not $ExamplesDir) {
    $ExamplesDir = Join-Path (Split-Path $Root) "wasmtime-android-kt-examples"
}
if (-not (Test-Path $ExamplesDir)) {
    throw "examples repo not found at $ExamplesDir. Clone wasmtime-android-kt-examples or set EXAMPLES_DIR."
}

$so = Join-Path (Join-Path (Join-Path $Root "android") "jniLibs") "arm64-v8a/libwasmtime_android_kt.so"
if (-not (Test-Path $so)) {
    throw "missing $so — run scripts/build-native-android.ps1 first."
}

$hostDir = Join-Path (Join-Path $ExamplesDir "hosts") "fullscreen-surface"
$gradlew = Join-Path $hostDir "gradlew.bat"
if (-not (Test-Path $gradlew)) {
    $gradlew = Join-Path $hostDir "gradlew"
}
if (-not (Test-Path $gradlew)) {
    throw "missing Gradle wrapper under $hostDir"
}

$guest = Join-Path (Join-Path (Join-Path $ExamplesDir "guests") "rotating-cube") "dist/guest.wasm"
if (-not (Test-Path $guest)) {
    throw "missing cube guest $guest"
}

$sdkDir = $env:ANDROID_SDK_ROOT
if (-not $sdkDir) { $sdkDir = $env:ANDROID_HOME }
$ourProps = Join-Path $Root "local.properties"
if ((-not $sdkDir) -and (Test-Path $ourProps)) {
    foreach ($line in Get-Content -Path $ourProps) {
        if ($line -match '^\s*sdk\.dir\s*=\s*(.+)\s*$') {
            $sdkDir = $Matches[1].Trim() -replace '\\:', ':' -replace '\\\\', '\'
        }
    }
}
if (-not $sdkDir) {
    throw "set ANDROID_SDK_ROOT / ANDROID_HOME or sdk.dir in this repo's local.properties"
}

function Escape-JavaProperty([string]$value) {
    return ($value.Replace('\', '/'))
}

$propsPath = Join-Path $hostDir "local.properties"
$lines = @()
if (Test-Path $propsPath) {
    $lines = @(Get-Content -Path $propsPath)
}
$lines = @($lines | Where-Object { $_ -notmatch '^\s*wasmtime\.android\.kt\.dir\s*=' })
if (-not ($lines | Where-Object { $_ -match '^\s*sdk\.dir\s*=' })) {
    $lines += "sdk.dir=$(Escape-JavaProperty $sdkDir)"
}
$lines += "wasmtime.android.kt.dir=$(Escape-JavaProperty $Root)"
Set-Content -Path $propsPath -Value $lines -Encoding ascii
Write-Host "Wrote $propsPath (includeBuild $($Root))"

$wantInstall = $false
if ($AssembleOnly) {
    $wantInstall = $false
} elseif ($Install) {
    $wantInstall = $true
} else {
    $adb = Get-Command adb -ErrorAction SilentlyContinue
    if ($adb) {
        $devices = & adb devices
        $wantInstall = [bool]($devices | Select-String -Pattern '\tdevice$')
    }
}

Push-Location $hostDir
try {
    & $gradlew :app:assembleDebug --no-daemon
    if ($LASTEXITCODE -ne 0) { throw "examples assembleDebug failed: $LASTEXITCODE" }
    if ($wantInstall) {
        & $gradlew :app:installDebug --no-daemon
        if ($LASTEXITCODE -ne 0) { throw "examples installDebug failed: $LASTEXITCODE" }
        Write-Host "installed fullscreen-surface (rotating-cube guest) on the attached device"
    } else {
        Write-Host "assembleDebug OK (no device / -AssembleOnly: skipped installDebug)"
    }
} finally {
    Pop-Location
}
