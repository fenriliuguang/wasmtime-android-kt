#!/usr/bin/env python3
"""Next 0.1.0 product-gate PR. Same output as product-010-remaining.ps1.

P0 wasi:webgpu and P1 WASI 0.3 auto queues are closed. P2 Wasmtime pin is
named-only here. This script prints P010-* from needles in
docs/scheme/product-010.md.
"""
from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TRACKING = ROOT / "docs" / "scheme" / "product-010.md"

LANES = (
    (
        "P010-SPI",
        "sink-experimental-host-callbacks",
        "move ExperimentalHostCallbacks out of runtime public SPI",
        "gap: p010 spi pending",
    ),
    (
        "P010-DISC",
        "dual-track-store-factory",
        "Store discover factory; explicit setWebGpuBackend wins",
        "gap: p010 disc pending",
    ),
    (
        "P010-FIX",
        "fixtures-out-of-product-linker",
        "drop get-gpu/get-device from the product linker",
        "gap: p010 fix pending",
    ),
    (
        "P010-CLIERR",
        "cli-product-errors",
        "guest-visible error-code on product cli stdio/run",
        "gap: p010 cli-err pending",
    ),
    (
        "P010-TCP",
        "outbound-tcp",
        "connect(ip-socket-address) dials non-loopback IPv4",
        "gap: p010 tcp pending",
    ),
    (
        "P010-HBODY",
        "http-body-stream",
        "HTTP body stream<u8> on request/response",
        "gap: p010 http-body pending",
    ),
    (
        "P010-HOUT",
        "http-outbound",
        "outgoing-handler / send (not in-process 200-only)",
        "gap: p010 http-out pending",
    ),
    (
        "P010-HCTOR",
        "http-drop-product-ctors",
        "drop product [constructor]request/response",
        "gap: p010 http-ctor pending",
    ),
    (
        "P010-GFXP",
        "vendor-wasi-gfx-wit",
        "vendor one dated wasi-gfx WIT under third_party/",
        "gap: p010 gfx-pin pending",
    ),
    (
        "P010-GFXH",
        "gfx-on-frame-host",
        "host surface + on-frame CM stream on GpuThread",
        "gap: p010 gfx-host pending",
    ),
    (
        "P010-GFXL",
        "gfx-product-frame-loop",
        "product guest on-frame loop + multi-frame device instrument",
        "gap: p010 gfx-loop pending",
    ),
    (
        "P010-GFXB",
        "gfx-product-adapter-device",
        "frame loop uses product request-adapter/request-device (no get-device)",
        "gap: p010 gfx-boot pending",
    ),
    (
        "P010-GFXV",
        "gfx-vsync-on-frame",
        "Choreographer vsync writes on-frame; drop unconsumed beats",
        "gap: p010 gfx-vsync pending",
    ),
    (
        "P010-DEMO",
        "device-and-out-of-tree-demo",
        "named device on-screen row + README link to out-of-tree wasm demo",
        "gap: p010 demo pending",
    ),
    (
        "P010-CLAIM",
        "010-claim-table",
        "release-notes claim table (most-of-pin; not CTS)",
        "gap: p010 claim pending",
    ),
    (
        "P010-PUB",
        "publish-010",
        "Central + Packages CI; version 0.1.0",
        "gap: p010 publish pending",
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

    print("Playbook: docs/agent/product-010.md")
    if leftover:
        lane_id, title, method = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
        print("Tracking: docs/scheme/product-010.md")
    else:
        print("Next: (0.1.0 product queue empty)")
        print("Tracking: docs/scheme/product-010.md")
        print("Living auto: native-dawn — python3 ./scripts/native-dawn-remaining.py")
        print(
            "Named-only: P2 Wasmtime pin, G-cmd, G-fs-full, listen/UDP, "
            "wasi-testsuite, wasmtime-wasi, this-repo 1.0 — do not auto-cut; "
            "never file upstream issues."
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
