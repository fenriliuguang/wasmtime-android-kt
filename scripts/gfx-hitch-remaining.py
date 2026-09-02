#!/usr/bin/env python3
"""Next cube-hitch restart *commit* on fix/300-gfx-cube-pop.

Native-dawn consume leftover is empty. This script prints HP-* from needles
in docs/scheme/gfx-hitch.md. Do not inherit mapping §§0–5 Closed rows.
"""
from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TRACKING = ROOT / "docs" / "scheme" / "gfx-hitch.md"

LANES = (
    (
        "HP-LOG",
        "device-hotpath-window",
        ">=2 min GfxHitch hotpath + hotpath-spike on V2458A; record mapping 6.6",
        "gap: hitch log pending",
    ),
    (
        "HP-BIND",
        "bind-eye-pop-to-stage",
        "one sentence: stage spike or no in-process spike at the pop",
        "gap: hitch bind pending",
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

    print("Playbook: docs/agent/gfx-hitch.md")
    print("Branch: fix/300-gfx-cube-pop")
    if leftover:
        lane_id, title, method = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
        print("Tracking: docs/scheme/gfx-hitch.md")
        print("Do not open a PR; commit this lane on fix/300-gfx-cube-pop.")
        print("Forget inherited Closed/Likely; read mapping section 6 hot-path stages.")
    else:
        print("Next: (gfx hitch restart queue empty)")
        print("Tracking: docs/scheme/gfx-hitch.md")
        print(
            "Named-only follow-up: compositor/acquire, guest/CM, "
            "GPU fence vs D24, event screenrecord — user must name one. "
            "Do not restack keep/DisplayManager/GameState/JNI."
        )
        print("Never file upstream issues.")

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
