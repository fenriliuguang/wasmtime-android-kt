#!/usr/bin/env python3
"""Next Dawn-consume / WG-6 leftover PR. Same output as webgpu-guest-dawn-remaining.ps1."""
from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JVM = ROOT / "native/src/jvm.rs"
DAWN = (
    ROOT
    / "host-dawn/src/main/kotlin/io/github/fenriliuguang/wasi/webgpu/experimental/dawn/DawnWasiWebGpuHost.kt"
)
HOST_TYPES = (
    ROOT
    / "host-dawn/src/main/kotlin/io/github/fenriliuguang/wasi/webgpu/experimental/host/HostTypes.kt"
)

G1_SENTINEL = """\
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
"""

G2_SENTINEL = """\
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
"""

G3_SENTINEL = """\
            GPUShaderModuleDescriptor(
                label = descriptor.label,
                shaderSourceWGSL = GPUShaderSourceWGSL(descriptor.code),
            ),
"""

G4_SENTINEL = """\
            forceFallbackAdapter = options.forceFallbackAdapter,
            // Android Surface path needs Vulkan; Undefined may pick GLES and leave the
            // native window connected, so CM Vulkan createSurface hits WINDOW_IN_USE.
            backendType = BackendType.Vulkan,
        )
"""

G5_SENTINEL = """\
            requiredLimits = dawnRequiredLimits(descriptor.requiredLimits),
            deviceLostCallbackExecutor = callbackExecutor,
"""

G6_SENTINEL = """\
data class ColorTargetState(
    val format: Int,
    val blend: BlendState? = null,
)
"""

G7_SENTINEL = """\
data class DepthStencilState(
    val format: Int,
    val depthWriteEnabled: Boolean = true,
    /** Dawn CompareFunction pass-through ([GpuCompareFunction]). */
    val depthCompare: Int = GpuCompareFunction.LESS,
)
"""

G9_SENTINEL = "auto pipeline layout; pass an explicit pipeline-layout handle"

DAWN_LANES = (
    (
        "G1",
        "dawn-consume-render-pipeline-extras",
        "Dawn GPUPrimitiveState / GPUColorTargetState / multisample",
        G1_SENTINEL,
    ),
    (
        "G2",
        "dawn-consume-texture-view-formats",
        "Dawn GPUTextureDescriptor viewFormats",
        G2_SENTINEL,
    ),
    (
        "G3",
        "dawn-consume-shader-compilation-hints",
        "Dawn GPUShaderModuleDescriptor compilationHints",
        G3_SENTINEL,
    ),
    (
        "G4",
        "dawn-consume-xr-compatible",
        "Dawn GPURequestAdapterOptions xrCompatible",
        G4_SENTINEL,
    ),
    (
        "G5",
        "dawn-consume-default-queue",
        "Dawn GPUDeviceDescriptor defaultQueue",
        G5_SENTINEL,
    ),
)

HOST_LANES = (
    (
        "G6",
        "color-target-write-mask",
        "[method]gpu-device.create-render-pipeline write-mask",
        G6_SENTINEL,
    ),
    (
        "G7",
        "depth-stencil-leftovers",
        "[method]gpu-device.create-render-pipeline depth-stencil leftovers",
        G7_SENTINEL,
    ),
)


def jni_sig(text: str, name: str) -> str | None:
    m = re.search(rf'"{re.escape(name)}",\s*\n\s*"([^"]+)"', text)
    return m.group(1) if m else None


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    for path in (JVM, DAWN, HOST_TYPES):
        if not path.is_file():
            raise SystemExit(f"missing {path}")
    jvm = JVM.read_text(encoding="utf-8").replace("\r\n", "\n")
    dawn = DAWN.read_text(encoding="utf-8").replace("\r\n", "\n")
    host_types = HOST_TYPES.read_text(encoding="utf-8").replace("\r\n", "\n")

    leftover: list[tuple[str, str, str, str]] = []
    for lane_id, title, method, sentinel in DAWN_LANES:
        if sentinel in dawn:
            leftover.append((lane_id, title, method, "Dawn sentinel"))
    for lane_id, title, method, sentinel in HOST_LANES:
        if sentinel in host_types:
            leftover.append((lane_id, title, method, "HostTypes sentinel"))
    got = jni_sig(jvm, "canvasContextConfigureDescribed")
    if got == "(IIII)I":
        leftover.append(
            (
                "G8",
                "canvas-configuration-leftovers",
                "[method]gpu-canvas-context.configure leftovers",
                "JNI (IIII)I",
            )
        )
    if G9_SENTINEL in dawn:
        leftover.append(
            (
                "G9",
                "auto-pipeline-layout",
                "create-compute-pipeline layout auto",
                "auto pipeline layout throw",
            )
        )

    print("Playbook: docs/agent/webgpu-guest-dawn.md")
    if leftover:
        lane_id, title, method, _ = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
    else:
        print("Next: (G1–G9 empty)")
        print(
            "Named-only: WG-6 real guest compute, WG-6 real guest 3D, "
            "canvas present guest-drawn frame, stage-only required-limits keys — "
            "do not auto-cut; never file upstream issues."
        )

    if not args.all:
        return
    print()
    if not leftover:
        print("=== (none) ===")
        return
    for lane_id, title, method, why in leftover:
        print(f"=== {lane_id} {title} ({why}) ===")
        print(f"  {method}")


if __name__ == "__main__":
    main()
