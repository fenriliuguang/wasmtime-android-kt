#!/usr/bin/env python3
"""Next NativeGpu Remaining Table PR (gap-webgpu-native-dawn.md §3).

Not WASI leftover. Not CTS. Not Record holes / gfx named-only.
Do not treat wasi-p3-leftover-remaining.py empty as “nothing to cut”.
"""
from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TRACKING = ROOT / "docs" / "scheme" / "nativegpu-remaining.md"

LANES = (
    (
        "N-COMPINFO",
        "compilation-info",
        "compilation-info / messages from wgpuShaderModuleGetCompilationInfo",
        "gap: n-compinfo pending",
    ),
    (
        "N-POPERR",
        "pop-error-scope",
        "pop-error-scope returns callback type/message, not always ok(none)",
        "gap: n-poperr pending",
    ),
    (
        "N-UNCAPTURED",
        "uncaptured-error",
        "on-uncaptured-error + gpu-error / device-lost-info from Dawn callbacks",
        "gap: n-uncaptured pending",
    ),
    (
        "N-WGSLFEAT",
        "wgsl-language-features",
        "wgsl-language-features.has queries Dawn (not always false)",
        "gap: n-wgslfeat pending",
    ),
    (
        "N-LABELS",
        "object-labels",
        "wgpu*SetLabel / get for resources other than buffer/texture",
        "gap: n-labels pending",
    ),
    (
        "N-IMMEDIATE",
        "immediates-debug",
        "set-immediates / debug group·marker C wrappers",
        "gap: n-immediate pending",
    ),
    (
        "N-FENCE",
        "submit-fence",
        "queue.submit canvas recycle waits on a fence vs onSubmittedWorkDone",
        "gap: n-fence pending",
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

    print("Playbook: docs/scheme/nativegpu-remaining.md")
    print("Queue: NativeGpu Remaining Table (not WASI leftover)")
    if leftover:
        lane_id, title, method = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
        print("Tracking: docs/scheme/nativegpu-remaining.md")
    else:
        print("Next: (NativeGpu Remaining Table empty)")
        print("Tracking: docs/scheme/nativegpu-remaining.md")
        print(
            "Named-only: Record holes, required-limits on request-device, "
            "gfx unconfigure / timestamped frame-event / Lost-Outdated / "
            "multi-window, dawn-jni, WASI leftover, CTS — do not auto-cut."
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
