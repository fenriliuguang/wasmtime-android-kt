#!/usr/bin/env python3
"""Next WASI 0.3 (P1) PR. Same output as wasi-p3-remaining.ps1."""
from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WASI = ROOT / "fixtures" / "wasi"
P3 = ROOT / "fixtures" / "p3"

LANES = (
    (
        "W1",
        "stream-multi-chunk",
        "stream<T> multi-chunk / backpressure",
        P3 / "stream_chunks.wat",
        None,
        None,
    ),
    (
        "W2",
        "clocks-official-instant",
        "wasi:clocks/system-clock now official instant",
        WASI / "system_now.wat",
        "not official instant",
        None,
    ),
    (
        "W3",
        "cli-stdout-stderr-result",
        "wasi:cli stdout/stderr write-via-stream official result",
        WASI / "cli_stdout.wat",
        "future<u32> byte count",
        None,
    ),
    (
        "W4",
        "cli-stdin-tuple",
        "wasi:cli stdin read-via-stream official tuple",
        WASI / "cli_stdin.wat",
        "func() -> stream<u8>",
        None,
    ),
    (
        "W5",
        "cli-command-result",
        "wasi:cli/command official run result",
        WASI / "cli_command.wat",
        "official empty result deferred",
        None,
    ),
    (
        "W6",
        "filesystem-preopen",
        "wasi:filesystem Android sandbox preopen",
        WASI / "filesystem_preopen.wat",
        None,
        None,
    ),
    (
        "W7",
        "sockets-tcp",
        "wasi:sockets Android subset",
        WASI / "sockets_tcp.wat",
        None,
        None,
    ),
    (
        "W8",
        "http-handler",
        "wasi:http Android subset",
        WASI / "http_handler.wat",
        None,
        None,
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
    for lane_id, title, method, path, needle, _ in LANES:
        if active(path, needle):
            leftover.append((lane_id, title, method))

    print("Playbook: docs/agent/wasi-p3.md")
    if leftover:
        lane_id, title, method = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
    else:
        print("Next: (W1–W8 empty)")
        print(
            "Named-only: WASI 0.2 polyfill, full wasi-testsuite, "
            "wasmtime-wasi crate — do not auto-cut; never file upstream issues."
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
