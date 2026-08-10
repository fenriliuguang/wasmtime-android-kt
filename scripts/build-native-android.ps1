# Cross-compile Track B cdylib for Android and install into android/jniLibs.
# Aligns NDK / API / ABI / Rust toolchain with Track A build-wasmtime4j-android.ps1.
param(
    [string]$NdkVersion = "28.2.13676358",
    [int]$ApiLevel = 24,
    [string[]]$Targets = @("arm64-v8a", "x86_64"),
    [switch]$SkipStrip
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Native = Join-Path $Root "native"
$Out = Join-Path $Root "android\jniLibs"
$Stubs = Join-Path $Native "link-stubs"

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
    $env:RUSTUP_TOOLCHAIN = "1.97.1"
}
# Windows host + aarch64/x86_64-linux-android: rustc 1.97.1 ACCESS_VIOLATION at opt-level>=1.
if (($IsWindows -or $env:OS -eq "Windows_NT") -and -not $env:CARGO_PROFILE_RELEASE_OPT_LEVEL) {
    $env:CARGO_PROFILE_RELEASE_OPT_LEVEL = "0"
    Write-Host "Note: CARGO_PROFILE_RELEASE_OPT_LEVEL=0 (Windows Android cross-compile rustc workaround)"
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
Get-ChildItem -Recurse $Out -Filter "libwasmtime_android_kt.so" | ForEach-Object {
    if (-not $SkipStrip) {
        $strip = Join-Path $Ndk "toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-strip.exe"
        if (-not (Test-Path $strip)) {
            $strip = Get-ChildItem -Path (Join-Path $Ndk "toolchains\llvm\prebuilt") -Recurse -Filter "llvm-strip*" -ErrorAction SilentlyContinue |
                Select-Object -First 1 -ExpandProperty FullName
        }
        if ($strip -and (Test-Path $strip)) {
            & $strip --strip-unneeded $_.FullName
        }
    }
    "{0} ({1:N1} MB)" -f $_.FullName, ($_.Length / 1MB)
}
