#!/usr/bin/env python3
"""Next Wasmtime pin (P2) PR. Same output as wasmtime-p2-remaining.ps1.

P0 wasi:webgpu and P1 WASI 0.3 auto queues are closed. This script only
prints P2-EVAL / P2-PATCH from needles in docs/scheme/wasmtime-tracking.md.
Named leftovers (major RFC, wasmtime-wasi, P1 WIT gaps) are never Next:.
"""
from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TRACKING = ROOT / "docs" / "scheme" / "wasmtime-tracking.md"

LANES = (
    (
        "P2-EVAL",
        "wasmtime-pin-eval",
        "refresh wasmtime-tracking §2/§3; do not land major",
        TRACKING,
        "gap: p2 pin eval pending",
    ),
    (
        "P2-PATCH",
        "wasmtime-patch",
        "wasmtime 47.0.2 → 47.0.x per tracking §4.1",
        TRACKING,
        "gap: p2 patch pending",
    ),
)


def active(path: Path, needle: str | None) -> bool:
    if not path.is_file():
        return True
    if needle is None:
        return False
    text = path.read_text(encoding="utf-8").replace("\r\n", "\n")
    return needle in text


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    leftover: list[tuple[str, str, str]] = []
    for lane_id, title, method, path, needle in LANES:
        if active(path, needle):
            leftover.append((lane_id, title, method))

    print("Playbook: docs/agent/wasmtime-p2.md")
    if leftover:
        lane_id, title, method = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
        print("Tracking: docs/scheme/wasmtime-tracking.md")
    else:
        print("Next: (P2 pin eval empty; P2 patch empty)")
        print("Tracking: docs/scheme/wasmtime-tracking.md")
        print(
            "Named-only: major upgrade RFC, wasmtime-wasi crate, "
            "P1 leftover WASI shapes (docs/mapping/gap-wasi-p3-wit.md) "
            "— do not auto-cut; never file upstream issues."
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
