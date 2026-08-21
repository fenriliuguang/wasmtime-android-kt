#!/usr/bin/env python3
"""Next midterm-lane PR. Same output as webgpu-midterm-remaining.ps1. Use if pwsh is missing."""
from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WIT = ROOT / "third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit"
CM = ROOT / "native/src/cm.rs"

CANVAS = (
    "gpu-canvas-context.configure",
    "gpu-canvas-context.unconfigure",
    "gpu-canvas-context.get-configuration",
    "gpu-canvas-context.get-current-texture",
)


def wrap_bodies(text: str) -> dict[str, str]:
    lines = text.splitlines()
    bodies: dict[str, str] = {}
    i = 0
    n = len(lines)
    while i < n:
        if not re.search(r"func_wrap(_concurrent)?\(", lines[i]):
            i += 1
            continue
        name = None
        for j in range(i, min(i + 8, n - 1) + 1):
            m = re.search(r'"\[method\]([^"]+)"', lines[j])
            if m:
                name = m.group(1)
                break
        if name is None:
            i += 1
            continue
        depth = 0
        started = False
        buf: list[str] = []
        for k in range(i, n):
            line = lines[k]
            buf.append(line)
            for ch in line:
                if ch == "(":
                    depth += 1
                    started = True
                elif ch == ")":
                    depth -= 1
            if started and depth <= 0 and k > i:
                break
        bodies[name] = "\n".join(buf)
        i += 1
    return bodies


def kind(bodies: dict[str, str], short: str) -> str:
    if short not in bodies:
        return "unhung"
    body = bodies[short]
    if "_described" in body:
        return "described"
    if "jvm::exp_" in body:
        return "host-fixed"
    return "lift-only"


def needs_l2(bodies: dict[str, str], short: str) -> bool:
    return kind(bodies, short) in ("unhung", "lift-only", "host-fixed")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    ap.add_argument("-IncludeRecords", "--include-records", action="store_true")
    args = ap.parse_args()
    if not WIT.is_file() or not CM.is_file():
        raise SystemExit(f"missing {WIT} or {CM}")
    bodies = wrap_bodies(CM.read_text(encoding="utf-8"))

    a1 = [n for n in CANVAS if kind(bodies, n) == "unhung"]
    a2 = [
        n
        for n in ("gpu-canvas-context.configure",)
        if needs_l2(bodies, n) and kind(bodies, n) != "unhung"
    ]
    a3core = [
        n
        for n in ("gpu-canvas-context.get-current-texture",)
        if needs_l2(bodies, n) and kind(bodies, n) != "unhung"
    ]
    a3ride = [
        n
        for n in ("gpu-canvas-context.unconfigure",)
        if needs_l2(bodies, n) and kind(bodies, n) != "unhung"
    ]
    a4 = [
        n
        for n in ("gpu-canvas-context.get-configuration",)
        if needs_l2(bodies, n) and kind(bodies, n) != "unhung"
    ]
    b1 = [n for n in ("gpu-device.queue",) if kind(bodies, n) == "host-fixed"]
    b2 = [n for n in ("gpu-adapter.request-device",) if kind(bodies, n) == "host-fixed"]
    b3 = [n for n in ("gpu.request-adapter",) if kind(bodies, n) == "host-fixed"]
    records = sorted(k for k in bodies if k.startswith("record-"))
    c = [n for n in records if kind(bodies, n) != "described"]

    next_id = next_title = None
    next_names: list[str] = []
    if a1:
        next_id, next_title, next_names = "A1", "canvas-shape", a1
    elif a2:
        next_id, next_title, next_names = "A2", "canvas-configure-L2", a2
    elif a3core or a3ride:
        next_id, next_title, next_names = "A3", "canvas-current-texture-L2", a3core + a3ride
    elif a4:
        next_id, next_title, next_names = "A4", "canvas-get-configuration-L2", a4
    elif b1:
        next_id, next_title, next_names = "B1", "device-queue-rep", b1
    elif b2:
        next_id, next_title, next_names = "B2", "request-device-descriptor", b2
    elif b3:
        next_id, next_title, next_names = "B3", "request-adapter-options", b3
    elif args.include_records and c:
        first_res = c[0].split(".", 1)[0]
        mutate = [n for n in c if n.startswith(first_res + ".") and n.rsplit(".", 1)[-1] in ("add", "get", "has", "remove")]
        if mutate:
            next_id, next_title, next_names = "C", f"{first_res}-mutate", mutate
        else:
            rest = [n for n in c if n.startswith(first_res + ".")]
            next_id, next_title, next_names = "C", f"{first_res}-iterate", rest

    print("Playbook: docs/agent/webgpu-midterm.md")
    if next_id is None:
        print("Next: (A–C empty)")
        print("Lane D is manual (真机 / WG-5 / cite) — do not auto-cut; never file upstream issues.")
    else:
        print(f"Next: {next_id} {next_title}")
        for n in next_names:
            print(f"  [method]{n}")

    if not args.all:
        return
    print()
    print("=== A1 canvas unhung ===")
    for n in a1:
        print(f"  [method]{n}")
    print("=== A2 configure needs L2 ===")
    for n in a2:
        print(f"  [method]{n}")
    print("=== A3 current-texture / unconfigure needs L2 ===")
    for n in a3core + a3ride:
        print(f"  [method]{n}")
    print("=== A4 get-configuration needs L2 ===")
    for n in a4:
        print(f"  [method]{n}")
    print("=== B S1–S3 host-fixed ===")
    for n in b1 + b2 + b3:
        print(f"  [method]{n}")
    if args.include_records:
        print("=== C record-* not described ===")
        for n in c:
            print(f"  [method]{n}")
    else:
        print("=== C record-* omitted (pass -IncludeRecords) ===")
    print("=== D ===")
    print("  manual cite only; never file upstream issues")


if __name__ == "__main__":
    main()
