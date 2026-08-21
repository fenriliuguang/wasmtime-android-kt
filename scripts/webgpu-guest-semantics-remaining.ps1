# Next leftover-descriptor-semantics PR. Agents: run this instead of grepping cm.rs or reading RFCs.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Jvm = Join-Path $Root "native\src\jvm.rs"
$Dawn = Join-Path $Root "host-dawn\src\main\kotlin\io\github\fenriliuguang\wasi\webgpu\experimental\dawn\DawnWasiWebGpuHost.kt"
if (-not (Test-Path $Jvm)) { throw "missing $Jvm" }
if (-not (Test-Path $Dawn)) { throw "missing $Dawn" }
$jvmText = [System.IO.File]::ReadAllText($Jvm).Replace("`r`n", "`n")
$dawnText = [System.IO.File]::ReadAllText($Dawn).Replace("`r`n", "`n")

function Get-JniSig([string]$name) {
    $m = [regex]::Match($jvmText, [regex]::Escape('"' + $name + '"') + ',\s*\r?\n\s*"([^"]+)"')
    if ($m.Success) { return $m.Groups[1].Value }
    return $null
}

$jniLanes = @(
    @{ Id = "F1"; Title = "render-pipeline-blend-ms-cull"; Jni = "deviceCreateRenderPipelineDescribed"; Sig = "(IILjava/lang/String;ILjava/lang/String;IILjava/lang/String;[I[I[I[I[I[III)I"; Method = "[method]gpu-device.create-render-pipeline" }
    @{ Id = "F2"; Title = "begin-render-pass-color-list"; Jni = "beginRenderPassDescribed"; Sig = "(IIIIIFFFFIIIIF)I"; Method = "[method]gpu-command-encoder.begin-render-pass" }
    @{ Id = "F3"; Title = "create-texture-view-formats-label"; Jni = "deviceCreateTextureDescribed"; Sig = "(IIIIIIIII)I"; Method = "[method]gpu-device.create-texture" }
    @{ Id = "F4"; Title = "create-buffer-mapped-label"; Jni = "deviceCreateBufferDescribed"; Sig = "(IJI)I"; Method = "[method]gpu-device.create-buffer" }
    @{ Id = "F5"; Title = "create-shader-hints-label"; Jni = "deviceCreateShaderModuleDescribed"; Sig = "(ILjava/lang/String;)I"; Method = "[method]gpu-device.create-shader-module" }
    @{ Id = "F6"; Title = "request-adapter-xr-compatible"; Jni = "requestAdapterDescribed"; Sig = "(IILjava/lang/String;)I"; Method = "[method]gpu.request-adapter" }
    @{ Id = "F7"; Title = "request-device-default-queue"; Jni = "adapterRequestDeviceDescribed"; Sig = "(IIIILjava/lang/String;)I"; Method = "[method]gpu-adapter.request-device" }
)

$f8 = @"
                compute = GPUComputeState(
                    module = module,
                    entryPoint = descriptor.compute.entryPoint ?: "main",
                ),
"@
$f9 = @"
        val gpuDescriptor = GPUDeviceDescriptor(
            label = descriptor.label,
            deviceLostCallbackExecutor = callbackExecutor,
"@

$leftover = @()
foreach ($lane in $jniLanes) {
    $got = Get-JniSig $lane.Jni
    if ($got -eq $lane.Sig) {
        $leftover += @{ Id = $lane.Id; Title = $lane.Title; Method = $lane.Method; Why = "JNI $($lane.Sig)" }
    }
}
if ($dawnText.Contains($f8)) {
    $leftover += @{ Id = "F8"; Title = "dawn-consume-pipeline-constants"; Method = "Dawn GPUComputeState / vertex / fragment constants"; Why = "Dawn GPUComputeState omits constants" }
}
if ($dawnText.Contains($f9)) {
    $leftover += @{ Id = "F9"; Title = "dawn-consume-required-limits"; Method = "Dawn GPUDeviceDescriptor requiredLimits"; Why = "Dawn GPUDeviceDescriptor label+callbacks only" }
}

Write-Host "Playbook: docs/agent/webgpu-guest-semantics.md"
if ($leftover.Count -gt 0) {
    $n = $leftover[0]
    Write-Host "Next: $($n.Id) $($n.Title)"
    Write-Host "  $($n.Method)"
} else {
    Write-Host "Next: (F1–F9 empty)"
    Write-Host "Named-only: SupportedLimits handle-0, required-features full list — do not auto-cut; never file upstream issues."
}

if (-not $All) { exit 0 }

Write-Host ""
if ($leftover.Count -eq 0) {
    Write-Host "=== (none) ==="
} else {
    foreach ($n in $leftover) {
        Write-Host "=== $($n.Id) $($n.Title) ($($n.Why)) ==="
        Write-Host "  $($n.Method)"
    }
}
