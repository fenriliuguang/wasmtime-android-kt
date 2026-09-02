#!/usr/bin/env python3
"""Next remaining close-out commit.

Prints BIND / GFX-SIZE / GFX-PIN from needles in docs/scheme/remaining.md.
"""
from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TRACKING = ROOT / "docs" / "scheme" / "remaining.md"

LANES = (
    (
        "BIND",
        "dawn-c-full-bind",
        "remaining pin methods call webgpu.h when libwebgpu_dawn.so is loaded",
        "gap: remaining bind pending",
    ),
    (
        "GFX-SIZE",
        "surface-size-resize",
        "wasi-gfx surface height/width/request-set-size/on-resize",
        "gap: remaining gfx-size pending",
    ),
    (
        "GFX-PIN",
        "wasi-gfx-pin-streams",
        "remaining pin on-pointer-* / on-key-* streams",
        "gap: remaining gfx-pin pending",
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

    print("Playbook: docs/agent/remaining.md")
    if leftover:
        lane_id, title, method = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
        print("Tracking: docs/scheme/remaining.md")
    else:
        print("Next: (remaining close-out empty)")
        print("Tracking: docs/scheme/remaining.md")
        print(
            "Named-only: unconfigure, timestamped frame-event, "
            "Lost/Outdated result, multi-window, P2 Wasmtime pin, "
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
