# Next midterm-lane PR. Agents: run this instead of grepping cm.rs or reading RFCs.
# After default shape remaining 0 (canvas omitted) and default L2 host-fixed 0.
param(
    [switch]$All,
    [switch]$IncludeRecords
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Wit = Join-Path $Root "third_party\wasi-webgpu\v0.3.0-rc.2\wit\webgpu.wit"
$Cm = Join-Path $Root "native\src\cm.rs"

if (-not (Test-Path $Wit)) { throw "missing $Wit" }
if (-not (Test-Path $Cm)) { throw "missing $Cm" }

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

function Get-Kind([string]$short) {
    if (-not $bodies.ContainsKey($short)) { return "unhung" }
    $body = $bodies[$short]
    if ($body -match '_described') { return "described" }
    if ($body -match 'jvm::exp_') { return "host-fixed" }
    return "lift-only"
}

function Test-NeedsL2([string]$short) {
    $k = Get-Kind $short
    return ($k -eq "unhung" -or $k -eq "lift-only" -or $k -eq "host-fixed")
}

$canvas = @(
    "gpu-canvas-context.configure",
    "gpu-canvas-context.unconfigure",
    "gpu-canvas-context.get-configuration",
    "gpu-canvas-context.get-current-texture"
)
$a1 = @($canvas | Where-Object { (Get-Kind $_) -eq "unhung" })
$a2 = @("gpu-canvas-context.configure" | Where-Object { (Test-NeedsL2 $_) -and (Get-Kind $_) -ne "unhung" })
$a3core = @("gpu-canvas-context.get-current-texture" | Where-Object { (Test-NeedsL2 $_) -and (Get-Kind $_) -ne "unhung" })
$a3ride = @("gpu-canvas-context.unconfigure" | Where-Object { (Test-NeedsL2 $_) -and (Get-Kind $_) -ne "unhung" })
$a4 = @("gpu-canvas-context.get-configuration" | Where-Object { (Test-NeedsL2 $_) -and (Get-Kind $_) -ne "unhung" })

$b1 = @("gpu-device.queue" | Where-Object { (Get-Kind $_) -eq "host-fixed" })
$b2 = @("gpu-adapter.request-device" | Where-Object { (Get-Kind $_) -eq "host-fixed" })
$b3 = @("gpu.request-adapter" | Where-Object { (Get-Kind $_) -eq "host-fixed" })

$records = @($bodies.Keys | Where-Object { $_ -like "record-*" } | Sort-Object)
$c = @($records | Where-Object { (Get-Kind $_) -ne "described" })

$nextId = $null
$nextTitle = $null
$nextNames = @()
if ($a1.Count -gt 0) {
    $nextId = "A1"; $nextTitle = "canvas-shape"; $nextNames = $a1
} elseif ($a2.Count -gt 0) {
    $nextId = "A2"; $nextTitle = "canvas-configure-L2"; $nextNames = $a2
} elseif ($a3core.Count -gt 0 -or $a3ride.Count -gt 0) {
    $nextId = "A3"; $nextTitle = "canvas-current-texture-L2"
    $nextNames = @($a3core + $a3ride)
} elseif ($a4.Count -gt 0) {
    $nextId = "A4"; $nextTitle = "canvas-get-configuration-L2"; $nextNames = $a4
} elseif ($b1.Count -gt 0) {
    $nextId = "B1"; $nextTitle = "device-queue-rep"; $nextNames = $b1
} elseif ($b2.Count -gt 0) {
    $nextId = "B2"; $nextTitle = "request-device-descriptor"; $nextNames = $b2
} elseif ($b3.Count -gt 0) {
    $nextId = "B3"; $nextTitle = "request-adapter-options"; $nextNames = $b3
} elseif ($IncludeRecords -and $c.Count -gt 0) {
    $nextId = "C"
    $firstRes = ($c[0] -split "\.")[0]
    $nextTitle = "$firstRes-mutate"
    $nextNames = @($c | Where-Object { $_.StartsWith("$firstRes.") -and ($_ -match '\.(add|get|has|remove)$') })
    if ($nextNames.Count -eq 0) {
        $nextTitle = "$firstRes-iterate"
        $nextNames = @($c | Where-Object { $_.StartsWith("$firstRes.") })
    }
}

Write-Host "Playbook: docs/agent/webgpu-midterm.md"
if ($null -eq $nextId) {
    Write-Host "Next: (A–C empty)"
    Write-Host "Lane D is manual (真机 / WG-5 / cite) — do not auto-cut; never file upstream issues."
} else {
    Write-Host "Next: $nextId $nextTitle"
    foreach ($n in $nextNames) { Write-Host "  [method]$n" }
}

if (-not $All) { exit 0 }

Write-Host ""
Write-Host "=== A1 canvas unhung ==="
$a1 | ForEach-Object { Write-Host "  [method]$_" }
Write-Host "=== A2 configure needs L2 ==="
$a2 | ForEach-Object { Write-Host "  [method]$_" }
Write-Host "=== A3 current-texture / unconfigure needs L2 ==="
@($a3core + $a3ride) | ForEach-Object { Write-Host "  [method]$_" }
Write-Host "=== A4 get-configuration needs L2 ==="
$a4 | ForEach-Object { Write-Host "  [method]$_" }
Write-Host "=== B S1–S3 host-fixed ==="
@($b1 + $b2 + $b3) | ForEach-Object { Write-Host "  [method]$_" }
if ($IncludeRecords) {
    Write-Host "=== C record-* not described ==="
    $c | ForEach-Object { Write-Host "  [method]$_" }
} else {
    Write-Host "=== C record-* omitted (pass -IncludeRecords) ==="
}
Write-Host "=== D ==="
Write-Host "  manual cite only; never file upstream issues"
