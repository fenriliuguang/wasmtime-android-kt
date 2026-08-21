# Next guest-pipeline PR. Agents: run this instead of grepping cm.rs or reading RFCs.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Jvm = Join-Path $Root "native\src\jvm.rs"
if (-not (Test-Path $Jvm)) { throw "missing $Jvm" }
$text = [System.IO.File]::ReadAllText($Jvm)

function Get-JniSig([string]$name) {
    $m = [regex]::Match($text, [regex]::Escape('"' + $name + '"') + ',\s*\r?\n\s*"([^"]+)"')
    if ($m.Success) { return $m.Groups[1].Value }
    return $null
}

$lanes = @(
    @{ Id = "P1"; Title = "bind-group-entries"; Jni = "deviceCreateBindGroupDescribed"; Sig = "(IILjava/lang/String;)I"; Method = "[method]gpu-device.create-bind-group" }
    @{ Id = "P2"; Title = "bind-group-layout-entries"; Jni = "deviceCreateBindGroupLayoutDescribed"; Sig = "(IIII)I"; Method = "[method]gpu-device.create-bind-group-layout" }
    @{ Id = "P3"; Title = "render-pipeline-vertex-buffers"; Jni = "deviceCreateRenderPipelineDescribed"; Sig = "(IILjava/lang/String;ILjava/lang/String;IILjava/lang/String;)I"; Method = "[method]gpu-device.create-render-pipeline" }
    @{ Id = "P4"; Title = "begin-render-pass-depth"; Jni = "beginRenderPassDescribed"; Sig = "(IIII)I"; Method = "[method]gpu-command-encoder.begin-render-pass" }
    @{ Id = "P5"; Title = "create-texture-mip-sample-dim"; Jni = "deviceCreateTextureDescribed"; Sig = "(IIIIII)I"; Method = "[method]gpu-device.create-texture" }
)

$leftover = @()
foreach ($lane in $lanes) {
    $got = Get-JniSig $lane.Jni
    if ($got -eq $lane.Sig) {
        $leftover += $lane
    }
}

Write-Host "Playbook closed: docs/agent/webgpu-guest-pipeline.md"
Write-Host "Use: .\scripts\webgpu-guest-semantics-remaining.ps1"
Write-Host "Playbook: docs/agent/webgpu-guest-pipeline.md"
if ($leftover.Count -gt 0) {
    $n = $leftover[0]
    Write-Host "Next: $($n.Id) $($n.Title)"
    Write-Host "  $($n.Method)"
} else {
    Write-Host "Next: (P1–P5 empty)"
    Write-Host "Named-only: sampler/view leftovers, pipeline constants, S1–S3 leftover fields, canvas present, Dawn render cite — do not auto-cut; never file upstream issues."
}

if (-not $All) { exit 0 }

Write-Host ""
if ($leftover.Count -eq 0) {
    Write-Host "=== (none) ==="
} else {
    foreach ($n in $leftover) {
        Write-Host "=== $($n.Id) $($n.Title) (JNI still $($n.Sig)) ==="
        Write-Host "  $($n.Method)"
    }
}
