# Next Dawn-consume / WG-6 leftover PR. Agents: run this instead of grepping cm.rs or reading RFCs.
param(
    [switch]$All
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Jvm = Join-Path $Root "native\src\jvm.rs"
$Dawn = Join-Path $Root "host-dawn\src\main\kotlin\io\github\fenriliuguang\wasi\webgpu\experimental\dawn\DawnWasiWebGpuHost.kt"
$HostTypes = Join-Path $Root "host-dawn\src\main\kotlin\io\github\fenriliuguang\wasi\webgpu\experimental\host\HostTypes.kt"
foreach ($p in @($Jvm, $Dawn, $HostTypes)) {
    if (-not (Test-Path $p)) { throw "missing $p" }
}
$jvmText = [System.IO.File]::ReadAllText($Jvm).Replace("`r`n", "`n")
$dawnText = [System.IO.File]::ReadAllText($Dawn).Replace("`r`n", "`n")
$hostText = [System.IO.File]::ReadAllText($HostTypes).Replace("`r`n", "`n")

function Get-JniSig([string]$name) {
    $m = [regex]::Match($jvmText, [regex]::Escape('"' + $name + '"') + ',\s*\r?\n\s*"([^"]+)"')
    if ($m.Success) { return $m.Groups[1].Value }
    return $null
}

$g1 = @"
                    // androidx GPUPrimitiveState extra ctor params (cull/front/strip)
                    // not assumed in this AAR; guest values stay on the Kotlin record (F1 DoD).
                    primitive = GPUPrimitiveState(topology = topology),
                    depthStencil = depthStencil,
                    fragment = GPUFragmentState(
                        module = fragmentModule,
                        entryPoint = descriptor.fragment.entryPoint ?: "fs_main",
                        constants = dawnPipelineConstants(descriptor.fragment.constants),
                        targets = descriptor.fragment.targets.map { target ->
                            GPUColorTargetState(format = target.format)
                        }.toTypedArray(),
                    ),
                    label = descriptor.label,
"@
$g2 = @"
            GPUTextureDescriptor(
                usage = descriptor.usage,
                size = GPUExtent3D(
                    width = descriptor.size.width,
                    height = descriptor.size.height,
                    depthOrArrayLayers = descriptor.size.depthOrArrayLayers,
                ),
                label = descriptor.label,
                dimension = descriptor.dimension,
                format = descriptor.format,
                mipLevelCount = descriptor.mipLevelCount,
                sampleCount = descriptor.sampleCount,
            ),
"@
$g3 = @"
            GPUShaderModuleDescriptor(
                label = descriptor.label,
                shaderSourceWGSL = GPUShaderSourceWGSL(descriptor.code),
            ),
"@
$g4 = @"
            forceFallbackAdapter = options.forceFallbackAdapter,
            // Android Surface path needs Vulkan; Undefined may pick GLES and leave the
            // native window connected, so CM Vulkan createSurface hits WINDOW_IN_USE.
            backendType = BackendType.Vulkan,
        )
"@
$g5 = @"
            requiredLimits = dawnRequiredLimits(descriptor.requiredLimits),
            deviceLostCallbackExecutor = callbackExecutor,
"@
$g6 = @"
data class ColorTargetState(
    val format: Int,
    val blend: BlendState? = null,
)
"@
$g7 = @"
data class DepthStencilState(
    val format: Int,
    val depthWriteEnabled: Boolean = true,
    /** Dawn CompareFunction pass-through ([GpuCompareFunction]). */
    val depthCompare: Int = GpuCompareFunction.LESS,
)
"@
$g9 = "auto pipeline layout; pass an explicit pipeline-layout handle"

$leftover = @()
$dawnLanes = @(
    @{ Id = "G1"; Title = "dawn-consume-render-pipeline-extras"; Method = "Dawn GPUPrimitiveState / GPUColorTargetState / multisample"; Sentinel = $g1 }
    @{ Id = "G2"; Title = "dawn-consume-texture-view-formats"; Method = "Dawn GPUTextureDescriptor viewFormats"; Sentinel = $g2 }
    @{ Id = "G3"; Title = "dawn-consume-shader-compilation-hints"; Method = "Dawn GPUShaderModuleDescriptor compilationHints"; Sentinel = $g3 }
    @{ Id = "G4"; Title = "dawn-consume-xr-compatible"; Method = "Dawn GPURequestAdapterOptions xrCompatible"; Sentinel = $g4 }
    @{ Id = "G5"; Title = "dawn-consume-default-queue"; Method = "Dawn GPUDeviceDescriptor defaultQueue"; Sentinel = $g5 }
)
foreach ($lane in $dawnLanes) {
    if ($dawnText.Contains($lane.Sentinel)) {
        $leftover += @{ Id = $lane.Id; Title = $lane.Title; Method = $lane.Method; Why = "Dawn sentinel" }
    }
}
$hostLanes = @(
    @{ Id = "G6"; Title = "color-target-write-mask"; Method = "[method]gpu-device.create-render-pipeline write-mask"; Sentinel = $g6 }
    @{ Id = "G7"; Title = "depth-stencil-leftovers"; Method = "[method]gpu-device.create-render-pipeline depth-stencil leftovers"; Sentinel = $g7 }
)
foreach ($lane in $hostLanes) {
    if ($hostText.Contains($lane.Sentinel)) {
        $leftover += @{ Id = $lane.Id; Title = $lane.Title; Method = $lane.Method; Why = "HostTypes sentinel" }
    }
}
$got = Get-JniSig "canvasContextConfigureDescribed"
if ($got -eq "(IIII)I") {
    $leftover += @{ Id = "G8"; Title = "canvas-configuration-leftovers"; Method = "[method]gpu-canvas-context.configure leftovers"; Why = "JNI (IIII)I" }
}
if ($dawnText.Contains($g9)) {
    $leftover += @{ Id = "G9"; Title = "auto-pipeline-layout"; Method = "create-compute-pipeline layout auto"; Why = "auto pipeline layout throw" }
}

Write-Host "Playbook: docs/agent/webgpu-guest-dawn.md"
if ($leftover.Count -gt 0) {
    $n = $leftover[0]
    Write-Host "Next: $($n.Id) $($n.Title)"
    Write-Host "  $($n.Method)"
} else {
    Write-Host "Next: (G1–G9 empty)"
    Write-Host "Named-only: WG-6 real guest compute, WG-6 real guest 3D, canvas present guest-drawn frame, stage-only required-limits keys — do not auto-cut; never file upstream issues."
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
