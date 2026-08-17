# Build wasmtime-android-kt cdylib for the host OS (desktop JVM convenience).
# Not a CI gate — Android ABI layout remains authoritative (docs/mapping/artifacts.md).
# Contributor flow: docs/contribute.md
param(
    [ValidateSet("release", "debug")]
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Native = Join-Path $Root "native"
$Out = Join-Path $Root "desktop\jniLibs"
$LoadLibrary = "wasmtime_android_kt"
$RustToolchain = if ($env:RUSTUP_TOOLCHAIN) { $env:RUSTUP_TOOLCHAIN } else { "1.97.1" }

if (-not (Test-Path (Join-Path $Native "Cargo.toml"))) {
    throw "Missing native crate at $Native"
}

$isWin = $IsWindows -or $env:OS -eq "Windows_NT"
if ($isWin) {
    $LibFile = "wasmtime_android_kt.dll"
    $HostOs = "windows"
} elseif ($IsMacOS) {
    $LibFile = "libwasmtime_android_kt.dylib"
    $HostOs = "macos"
} else {
    $LibFile = "libwasmtime_android_kt.so"
    $HostOs = "linux"
}

if (-not $env:RUSTUP_TOOLCHAIN) {
    $env:RUSTUP_TOOLCHAIN = $RustToolchain
}

New-Item -ItemType Directory -Force -Path $Out | Out-Null

$cargoArgs = @("build")
if ($Profile -eq "release") {
    $cargoArgs += "--release"
}

Push-Location $Native
try {
    $prevEap = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & cargo @cargoArgs
    $cargoExit = $LASTEXITCODE
    $ErrorActionPreference = $prevEap
    if ($cargoExit -ne 0) { throw "cargo build failed: $cargoExit" }
} finally {
    Pop-Location
}

$profileDir = if ($Profile -eq "release") { "release" } else { "debug" }
$built = Join-Path $Native "target\$profileDir\$LibFile"
if (-not (Test-Path $built)) {
    throw "expected host artifact missing after build: $built"
}

$dest = Join-Path $Out $LibFile
Copy-Item -Force -Path $built -Destination $dest
$item = Get-Item $dest
$sha = (Get-FileHash -Algorithm SHA256 -Path $dest).Hash.ToLowerInvariant()
"{0} ({1:N1} MB)" -f $item.FullName, ($item.Length / 1MB)

$info = [ordered]@{
    libraryFile   = $LibFile
    loadLibrary   = $LoadLibrary
    hostOs        = $HostOs
    profile       = $Profile
    rustToolchain = $RustToolchain
    builtAt       = (Get-Date).ToUniversalTime().ToString("o")
    bytes         = $item.Length
    sha256        = $sha
    note          = "Optional desktop shell only; Android jniLibs is the formal artifact layout."
}
$infoPath = Join-Path $Out "build-info.json"
($info | ConvertTo-Json -Depth 4) | Set-Content -Path $infoPath -Encoding utf8
Write-Host "Wrote $infoPath"
Write-Host "Desktop JVM: -Djava.library.path=$Out"
