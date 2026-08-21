#!/usr/bin/env python3
"""Next guest-pipeline PR. Same output as webgpu-guest-pipeline-remaining.ps1."""
from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JVM = ROOT / "native/src/jvm.rs"

# JNI name → (lane id, title) when this exact signature is still the described call.
LANES = (
    (
        "P1",
        "bind-group-entries",
        "deviceCreateBindGroupDescribed",
        "(IILjava/lang/String;)I",
        "[method]gpu-device.create-bind-group",
    ),
    (
        "P2",
        "bind-group-layout-entries",
        "deviceCreateBindGroupLayoutDescribed",
        "(IIII)I",
        "[method]gpu-device.create-bind-group-layout",
    ),
    (
        "P3",
        "render-pipeline-vertex-buffers",
        "deviceCreateRenderPipelineDescribed",
        "(IILjava/lang/String;ILjava/lang/String;IILjava/lang/String;)I",
        "[method]gpu-device.create-render-pipeline",
    ),
    (
        "P4",
        "begin-render-pass-depth",
        "beginRenderPassDescribed",
        "(IIII)I",
        "[method]gpu-command-encoder.begin-render-pass",
    ),
    (
        "P5",
        "create-texture-mip-sample-dim",
        "deviceCreateTextureDescribed",
        "(IIIIII)I",
        "[method]gpu-device.create-texture",
    ),
)


def jni_sig(text: str, name: str) -> str | None:
    m = re.search(rf'"{re.escape(name)}",\s*\n\s*"([^"]+)"', text)
    return m.group(1) if m else None


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    if not JVM.is_file():
        raise SystemExit(f"missing {JVM}")
    text = JVM.read_text(encoding="utf-8")

    leftover: list[tuple[str, str, str, str]] = []
    for lane_id, title, jni, sig, method in LANES:
        got = jni_sig(text, jni)
        if got == sig:
            leftover.append((lane_id, title, method, sig))

    print("Playbook closed: docs/agent/webgpu-guest-pipeline.md")
    print("Use: python ./scripts/webgpu-guest-semantics-remaining.py")
    print("Playbook: docs/agent/webgpu-guest-pipeline.md")
    if leftover:
        lane_id, title, method, _ = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
    else:
        print("Next: (P1–P5 empty)")
        print(
            "Named-only: sampler/view leftovers, pipeline constants, "
            "S1–S3 leftover fields, canvas present, Dawn render cite — "
            "do not auto-cut; never file upstream issues."
        )

    if not args.all:
        return
    print()
    for lane_id, title, method, sig in leftover:
        print(f"=== {lane_id} {title} (JNI still {sig}) ===")
        print(f"  {method}")
    if not leftover:
        print("=== (none) ===")


if __name__ == "__main__":
    main()
