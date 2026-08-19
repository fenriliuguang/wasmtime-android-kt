# Classify hung product [method] wraps: described L2 vs host-fixed L2 vs lift-only.
# Semantic-L2 agents: run this instead of reading cm.rs whole or scanning JNI by hand.
param(
    [switch]$IncludeAll,
    [switch]$IncludeCanvas
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Wit = Join-Path $Root "third_party\wasi-webgpu\v0.3.0-rc.2\wit\webgpu.wit"
$Cm = Join-Path $Root "native\src\cm.rs"

if (-not (Test-Path $Wit)) { throw "missing $Wit" }
if (-not (Test-Path $Cm)) { throw "missing $Cm" }

function Test-ProductCore([string]$fullName) {
    # S1/S2/S3 already product L2; not the default deepen queue.
    return @(
        "[method]gpu.request-adapter",
        "[method]gpu-adapter.request-device",
        "[method]gpu-device.queue"
    ) -contains $fullName
}

function Test-DefaultOmit([string]$fullName) {
    if ($fullName.StartsWith("[method]gpu-canvas-context.")) { return $true }
    if ($fullName.StartsWith("[method]record-")) { return $true }
    if ($fullName.StartsWith("[method]gpu-supported-limits.")) { return $true }
    if ($fullName.StartsWith("[method]gpu-adapter-info.")) { return $true }
    if ($fullName.StartsWith("[method]gpu-compilation-message.")) { return $true }
    if ($fullName -eq "[method]gpu-supported-features.has") { return $true }
    if ($fullName -eq "[method]wgsl-language-features.has") { return $true }
    if ($fullName -eq "[method]gpu-compilation-info.messages") { return $true }
    if ($fullName.StartsWith("[method]gpu-device-lost-info.")) { return $true }
    if ($fullName.StartsWith("[method]gpu-error.")) { return $true }
    if ($fullName -eq "[method]gpu-uncaptured-error-event.error") { return $true }
    if ($fullName -eq "[method]gpu.get-preferred-canvas-format") { return $true }
    if ($fullName -eq "[method]gpu.wgsl-language-features") { return $true }
    if ($fullName -match '\.(label|set-label)$') { return $true }
    return $false
}

$lines = [System.IO.File]::ReadAllLines($Cm)
$bodies = @{}
for ($i = 0; $i -lt $lines.Length; $i++) {
    if ($lines[$i] -notmatch 'func_wrap(_concurrent)?\(') { continue }
    $name = $null
    $endScan = [Math]::Min($i + 8, $lines.Length - 1)
    for ($j = $i; $j -le $endScan; $j++) {
        if ($lines[$j] -match '"\[method\]([^"]+)"') {
            $name = $Matches[1]
            break
        }
    }
    if ($null -eq $name) { continue }
    $depth = 0
    $started = $false
    $buf = New-Object System.Collections.Generic.List[string]
    for ($k = $i; $k -lt $lines.Length; $k++) {
        $line = $lines[$k]
        [void]$buf.Add($line)
        foreach ($ch in $line.ToCharArray()) {
            if ($ch -eq '(') { $depth++; $started = $true }
            elseif ($ch -eq ')') { $depth-- }
        }
        if ($started -and $depth -le 0 -and $k -gt $i) { break }
    }
    $bodies[$name] = ($buf -join "`n")
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
        $witMethods.Add("$resource.$($Matches[1])")
    }
}

$described = New-Object 'System.Collections.Generic.List[string]'
$hostFixed = New-Object 'System.Collections.Generic.List[string]'
$liftOnly = New-Object 'System.Collections.Generic.List[string]'
$unhung = New-Object 'System.Collections.Generic.List[string]'

foreach ($short in $witMethods) {
    $full = "[method]$short"
    if (-not $bodies.ContainsKey($short)) {
        [void]$unhung.Add($full)
        continue
    }
    $body = $bodies[$short]
    if ($body -match '_described') {
        [void]$described.Add($full)
    }
    elseif ($body -match 'jvm::exp_') {
        [void]$hostFixed.Add($full)
    }
    else {
        [void]$liftOnly.Add($full)
    }
}

function Select-Shown($list) {
    foreach ($name in $list) {
        if (-not $IncludeCanvas -and $name.StartsWith("[method]gpu-canvas-context.")) { continue }
        if (-not $IncludeAll -and (Test-DefaultOmit $name)) { continue }
        if (-not $IncludeAll -and (Test-ProductCore $name)) { continue }
        $name
    }
}

$shownHost = @(Select-Shown $hostFixed | Sort-Object)
$shownLift = @(Select-Shown $liftOnly | Sort-Object)
$shownUnhung = @(Select-Shown $unhung | Sort-Object)

Write-Host "WIT [method]: $($witMethods.Count)"
Write-Host "Wraps classified: $($bodies.Count)"
Write-Host "Described L2 (done): $($described.Count)"
Write-Host "Host-fixed L2: $($hostFixed.Count)"
Write-Host "Lift-only: $($liftOnly.Count)"
Write-Host "Unhung: $($unhung.Count)"
$omitNote = if ($IncludeAll) { "" } else { " (metadata/label/record omitted; pass -IncludeAll)" }
$canvasNote = if ($IncludeCanvas) { "" } else { " (canvas omitted; pass -IncludeCanvas)" }
Write-Host "Remaining host-fixed (prefer next): $($shownHost.Count)$omitNote$canvasNote"
Write-Host "Remaining lift-only: $($shownLift.Count)"
if ($shownUnhung.Count -gt 0) {
    Write-Host "Unhung (not a semantic-L2 cut): $($shownUnhung.Count)"
}

Write-Host ""
Write-Host "=== host-fixed (prefer; one [method] per PR) ==="
$shownHost
Write-Host ""
Write-Host "=== lift-only ==="
$shownLift
if ($shownUnhung.Count -gt 0) {
    Write-Host ""
    Write-Host "=== unhung ==="
    $shownUnhung
}
