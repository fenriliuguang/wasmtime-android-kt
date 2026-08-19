# Diff product [method] names in vendored WIT vs native/src/cm.rs wraps.
# Shape-slice agents: run this instead of downloading WIT or reading cm.rs whole.
param(
    [switch]$IncludeCanvas
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Wit = Join-Path $Root "third_party\wasi-webgpu\v0.3.0-rc.2\wit\webgpu.wit"
$Cm = Join-Path $Root "native\src\cm.rs"

if (-not (Test-Path $Wit)) { throw "missing $Wit" }
if (-not (Test-Path $Cm)) { throw "missing $Cm" }

$hung = New-Object 'System.Collections.Generic.HashSet[string]'
Select-String -Path $Cm -Pattern '"\[method\]([^"]+)"' -AllMatches | ForEach-Object {
    foreach ($m in $_.Matches) {
        [void]$hung.Add("[method]$($m.Groups[1].Value)")
    }
}

$resource = $null
$witMethods = New-Object 'System.Collections.Generic.List[string]'
foreach ($line in [System.IO.File]::ReadAllLines($Wit)) {
    if ($line -match '^\s+resource ([a-z0-9-]+) \{') {
        $resource = $Matches[1]
        continue
    }
    if ($null -ne $resource -and $line -match '^\s+\}$') {
        $resource = $null
        continue
    }
    if ($null -eq $resource) { continue }
    if ($line -match '^\s+constructor\(') { continue }
    if ($line -match '^\s+%?([a-z0-9-]+): (async )?func') {
        $witMethods.Add("[method]$resource.$($Matches[1])")
    }
}

$canvasPrefix = "[method]gpu-canvas-context."
$remaining = foreach ($name in $witMethods) {
    if ($hung.Contains($name)) { continue }
    if (-not $IncludeCanvas -and $name.StartsWith($canvasPrefix)) { continue }
    $name
}

$canvasSkip = @($witMethods | Where-Object { $_.StartsWith($canvasPrefix) -and -not $hung.Contains($_) }).Count

Write-Host "WIT [method]: $($witMethods.Count)"
Write-Host "Hung in cm.rs: $($hung.Count)"
Write-Host "Remaining: $($remaining.Count)$(if (-not $IncludeCanvas) { " (canvas omitted: $canvasSkip; pass -IncludeCanvas)" })"
$remaining | Sort-Object
