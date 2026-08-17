# Cross-compile wasmtime-android-kt cdylib for Android into android/jniLibs.
# Pins: docs/scheme/tech-stack.md · layout: docs/mapping/artifacts.md
param(
    [string]$NdkVersion = "28.2.13676358",
    [int]$ApiLevel = 24,
    [string[]]$Targets = @("arm64-v8a", "x86_64"),
    [switch]$SkipStrip,
    [switch]$SkipVerify
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Native = Join-Path $Root "native"
$Out = Join-Path $Root "android\jniLibs"
$Stubs = Join-Path $Native "link-stubs"
$LibFile = "libwasmtime_android_kt.so"
$LoadLibrary = "wasmtime_android_kt"
$RustToolchain = if ($env:RUSTUP_TOOLCHAIN) { $env:RUSTUP_TOOLCHAIN } else { "1.97.1" }

$Sdk = if ($env:ANDROID_SDK_ROOT) {
    $env:ANDROID_SDK_ROOT
} elseif ($env:ANDROID_HOME) {
    $env:ANDROID_HOME
} else {
    Join-Path $env:LOCALAPPDATA "Android\Sdk"
}
$Ndk = Join-Path $Sdk "ndk\$NdkVersion"
if (-not (Test-Path $Ndk)) {
    throw "Android NDK not found at $Ndk. Install with: sdkmanager --install `"ndk;$NdkVersion`"."
}

if (-not (Test-Path (Join-Path $Native "Cargo.toml"))) {
    throw "Missing native crate at $Native"
}

New-Item -ItemType Directory -Force -Path $Out | Out-Null
New-Item -ItemType Directory -Force -Path $Stubs | Out-Null
# Bionic has no libpthread; rustc still passes -lpthread on unix targets.
Set-Content -Path (Join-Path $Stubs "libpthread.so") -Value "INPUT(-lc)`n" -NoNewline -Encoding ascii

$env:ANDROID_NDK_HOME = $Ndk
$env:ANDROID_NDK_ROOT = $Ndk
if (-not $env:RUSTUP_TOOLCHAIN) {
    $env:RUSTUP_TOOLCHAIN = $RustToolchain
}
# Windows host + aarch64/x86_64-linux-android: rustc 1.97.1 can ACCESS_VIOLATION
# at some opt-levels. Default 2 so stream.write / cli stdio instrument frames
# fit ART (~1MiB) and the 8MiB cm-pump; override with 0 if rustc crashes.
if (($IsWindows -or $env:OS -eq "Windows_NT") -and -not $env:CARGO_PROFILE_RELEASE_OPT_LEVEL) {
    $env:CARGO_PROFILE_RELEASE_OPT_LEVEL = "2"
    Write-Host "Note: CARGO_PROFILE_RELEASE_OPT_LEVEL=2 (set 0 if rustc ACCESS_VIOLATION when cross-compiling)"
}
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS = "-Lnative=$Stubs"
$env:CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS = "-Lnative=$Stubs"

$ndkTargets = @()
foreach ($t in $Targets) {
    $ndkTargets += @("-t", $t)
}

Push-Location $Native
try {
    $prevEap = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & cargo ndk @ndkTargets -o $Out --platform $ApiLevel -- build --release
    $cargoExit = $LASTEXITCODE
    $ErrorActionPreference = $prevEap
    if ($cargoExit -ne 0) { throw "cargo ndk build failed: $cargoExit" }
} finally {
    Pop-Location
}

Write-Host "Installed Android natives under $Out"
$strip = $null
if (-not $SkipStrip) {
    $strip = Join-Path $Ndk "toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-strip.exe"
    if (-not (Test-Path $strip)) {
        $strip = Get-ChildItem -Path (Join-Path $Ndk "toolchains\llvm\prebuilt") -Recurse -Filter "llvm-strip*" -ErrorAction SilentlyContinue |
            Select-Object -First 1 -ExpandProperty FullName
    }
}

$abiMap = [ordered]@{}
foreach ($abi in $Targets) {
    $path = Join-Path $Out (Join-Path $abi $LibFile)
    if (-not (Test-Path $path)) {
        throw "expected artifact missing after build: $path"
    }
    if ($strip -and (Test-Path $strip)) {
        & $strip --strip-unneeded $path
    }
    $item = Get-Item $path
    $sha = (Get-FileHash -Algorithm SHA256 -Path $path).Hash.ToLowerInvariant()
    $abiMap[$abi] = [ordered]@{
        relativePath = "$abi/$LibFile"
        bytes        = $item.Length
        sha256       = $sha
    }
    "{0} ({1:N1} MB)" -f $item.FullName, ($item.Length / 1MB)
}

$info = [ordered]@{
    libraryFile   = $LibFile
    loadLibrary   = $LoadLibrary
    ndkVersion    = $NdkVersion
    apiLevel      = $ApiLevel
    rustToolchain = $RustToolchain
    builtAt       = (Get-Date).ToUniversalTime().ToString("o")
    abis          = $abiMap
}
$infoPath = Join-Path $Out "build-info.json"
($info | ConvertTo-Json -Depth 6) | Set-Content -Path $infoPath -Encoding utf8
Write-Host "Wrote $infoPath"

if (-not $SkipVerify) {
    & (Join-Path $PSScriptRoot "verify-native-android.ps1") -Abis $Targets
}
