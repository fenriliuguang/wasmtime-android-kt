#!/usr/bin/env python3
"""Next leftover-descriptor-semantics PR. Same output as webgpu-guest-semantics-remaining.ps1."""
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

# JNI name → leftover while this exact signature is still the described call.
JNI_LANES = (
    (
        "F1",
        "render-pipeline-blend-ms-cull",
        "deviceCreateRenderPipelineDescribed",
        "(IILjava/lang/String;ILjava/lang/String;IILjava/lang/String;[I[I[I[I[I[III)I",
        "[method]gpu-device.create-render-pipeline",
    ),
    (
        "F2",
        "begin-render-pass-color-list",
        "beginRenderPassDescribed",
        "(IIIIIFFFFIIIIF)I",
        "[method]gpu-command-encoder.begin-render-pass",
    ),
    (
        "F3",
        "create-texture-view-formats-label",
        "deviceCreateTextureDescribed",
        "(IIIIIIIII)I",
        "[method]gpu-device.create-texture",
    ),
    (
        "F4",
        "create-buffer-mapped-label",
        "deviceCreateBufferDescribed",
        "(IJI)I",
        "[method]gpu-device.create-buffer",
    ),
    (
        "F5",
        "create-shader-hints-label",
        "deviceCreateShaderModuleDescribed",
        "(ILjava/lang/String;)I",
        "[method]gpu-device.create-shader-module",
    ),
    (
        "F6",
        "request-adapter-xr-compatible",
        "requestAdapterDescribed",
        "(IILjava/lang/String;)I",
        "[method]gpu.request-adapter",
    ),
    (
        "F7",
        "request-device-default-queue",
        "adapterRequestDeviceDescribed",
        "(IIIILjava/lang/String;)I",
        "[method]gpu-adapter.request-device",
    ),
)

F8_SENTINEL = """\
                compute = GPUComputeState(
                    module = module,
                    entryPoint = descriptor.compute.entryPoint ?: "main",
                ),"""

F9_SENTINEL = """\
        val gpuDescriptor = GPUDeviceDescriptor(
            label = descriptor.label,
            deviceLostCallbackExecutor = callbackExecutor,"""


def jni_sig(text: str, name: str) -> str | None:
    m = re.search(rf'"{re.escape(name)}",\s*\n\s*"([^"]+)"', text)
    return m.group(1) if m else None


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    if not JVM.is_file():
        raise SystemExit(f"missing {JVM}")
    if not DAWN.is_file():
        raise SystemExit(f"missing {DAWN}")
    jvm = JVM.read_text(encoding="utf-8").replace("\r\n", "\n")
    dawn = DAWN.read_text(encoding="utf-8").replace("\r\n", "\n")

    leftover: list[tuple[str, str, str, str]] = []
    for lane_id, title, jni, sig, method in JNI_LANES:
        got = jni_sig(jvm, jni)
        if got == sig:
            leftover.append((lane_id, title, method, f"JNI {sig}"))

    if F8_SENTINEL in dawn:
        leftover.append(
            (
                "F8",
                "dawn-consume-pipeline-constants",
                "Dawn GPUComputeState / vertex / fragment constants",
                "Dawn GPUComputeState omits constants",
            )
        )
    if F9_SENTINEL in dawn:
        leftover.append(
            (
                "F9",
                "dawn-consume-required-limits",
                "Dawn GPUDeviceDescriptor requiredLimits",
                "Dawn GPUDeviceDescriptor label+callbacks only",
            )
        )

    print("Playbook: docs/agent/webgpu-guest-semantics.md")
    if leftover:
        lane_id, title, method, _ = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
    else:
        print("Next: (F1–F9 empty)")
        print(
            "Named-only: SupportedLimits handle-0, required-features full list — "
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
