# Verify Track B Android jniLibs layout (M5 artifact policy).
param(
    [string[]]$Abis = @("arm64-v8a", "x86_64"),
    [switch]$RequireAll,
    [long]$MinBytes = 1MB
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Out = Join-Path $Root "android\jniLibs"
$LibFile = "libwasmtime_android_kt.so"
$InfoPath = Join-Path $Out "build-info.json"

$required = if ($RequireAll) { @("arm64-v8a", "x86_64") } else { $Abis }
$info = $null
if (Test-Path $InfoPath) {
    $info = Get-Content -Raw -Path $InfoPath | ConvertFrom-Json
}

$failures = @()
foreach ($abi in $required) {
    $path = Join-Path $Out (Join-Path $abi $LibFile)
    if (-not (Test-Path $path)) {
        $failures += "missing: $path"
        continue
    }
    $item = Get-Item $path
    if ($item.Length -lt $MinBytes) {
        $failures += ("too small ({0} bytes < {1}): {2}" -f $item.Length, $MinBytes, $path)
        continue
    }
    if ($null -ne $info -and $null -ne $info.abis -and $info.abis.PSObject.Properties.Name -contains $abi) {
        $expected = [string]$info.abis.$abi.sha256
        if ($expected) {
            $hash = (Get-FileHash -Algorithm SHA256 -Path $path).Hash.ToLowerInvariant()
            if ($hash -ne $expected.ToLowerInvariant()) {
                $failures += "sha256 mismatch for $abi`: expected $expected got $hash"
            }
        }
    }
    Write-Host ("OK {0} ({1:N1} MB)" -f $path, ($item.Length / 1MB))
}

if ($failures.Count -gt 0) {
    Write-Error ("verify-native-android failed:`n  - " + ($failures -join "`n  - "))
    exit 1
}

Write-Host "verify-native-android: $($required.Count) ABI(s) OK"
