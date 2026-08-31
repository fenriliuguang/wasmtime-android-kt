#!/usr/bin/env python3
"""Next native-dawn host PR. Same output as native-dawn-remaining.ps1.

P0 wasi:webgpu shape, P1 WASI 0.3, and 0.1.0 product queues are closed.
This script prints ND-* from needles in docs/scheme/native-dawn.md.
"""
from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TRACKING = ROOT / "docs" / "scheme" / "native-dawn.md"

LANES = (
    (
        "ND-DISP",
        "dispatch-jni-vs-native",
        "cm.rs webgpu imports dispatch NativeGpu | JniBackend (JNI still default)",
        "gap: nd disp pending",
    ),
    (
        "ND-SO",
        "dawn-c-android-so",
        "one Dawn C API Android .so; changelog size + Android thread",
        "gap: nd so pending",
    ),
    (
        "ND-HOST",
        "native-gpu-trait-table",
        "Rust NativeGpu trait + handle table (DawnWasiWebGpuHost kinds)",
        "gap: nd host pending",
    ),
    (
        "ND-BOOT",
        "native-adapter-device-queue",
        "native request-adapter / request-device / queue + boot info",
        "gap: nd boot pending",
    ),
    (
        "ND-RES",
        "native-buffer-texture-shader-view",
        "native create-buffer/texture/sampler/shader/view + leftover records",
        "gap: nd res pending",
    ),
    (
        "ND-PIPE",
        "native-layouts-pipelines",
        "native bind-group/layout + compute/render pipelines (async + constants)",
        "gap: nd pipe pending",
    ),
    (
        "ND-ENC",
        "native-encoder-passes",
        "native command-encoder / render-compute pass / draws / copies",
        "gap: nd enc pending",
    ),
    (
        "ND-QUEUE",
        "native-queue-write-submit",
        "native submit / write-buffer-with-copy / write-texture / work-done (C API)",
        "gap: nd queue pending",
    ),
    (
        "ND-REST",
        "native-full-pin-method-suite",
        "remaining pin [method]s: wasi_webgpu_method green on NativeGpu",
        "gap: nd rest pending",
    ),
    (
        "ND-SURF",
        "native-canvas-anativewindow",
        "native Dawn Surface from ANativeWindow; hitch recycle invariants",
        "gap: nd surf pending",
    ),
    (
        "ND-DEFAULT",
        "default-backend-native",
        "product dawn() is NativeGpu; androidx leftover id dawn-jni; one .so",
        "gap: nd default pending",
    ),
    (
        "ND-CLAIM",
        "claim-native-dawn-consume",
        "claim-010 degree Dawn C (not JNI instantiate); still not CTS",
        "gap: nd claim pending",
    ),
    (
        "ND-DEVICE",
        "instruments-on-native-default",
        "existing gfx/WG-6 instruments on native; cube is demo row only",
        "gap: nd device pending",
    ),
)


def active(path: Path, needle: str) -> bool:
    if not path.is_file():
        return True
    text = path.read_text(encoding="utf-8").replace("\r\n", "\n")
    return needle in text


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    leftover: list[tuple[str, str, str]] = []
    for lane_id, title, method, needle in LANES:
        if active(TRACKING, needle):
            leftover.append((lane_id, title, method))

    print("Playbook: docs/agent/native-dawn.md")
    if leftover:
        lane_id, title, method = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
        print("Tracking: docs/scheme/native-dawn.md")
    else:
        print("Next: (native-dawn host queue empty)")
        print("Tracking: docs/scheme/native-dawn.md")
        print(
            "Named-only: P2 Wasmtime pin, cube-only path, hitch D3, "
            "G-cmd, G-fs-full, listen/UDP, wasi-testsuite, wasmtime-wasi, "
            "this-repo 1.0 — do not auto-cut; never file upstream issues."
        )

    if not args.all:
        return
    print()
    if not leftover:
        print("=== (none) ===")
        return
    for lane_id, title, method in leftover:
        print(f"=== {lane_id} {title} ===")
        print(f"  {method}")


if __name__ == "__main__":
    main()
