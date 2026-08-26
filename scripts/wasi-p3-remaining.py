#!/usr/bin/env python3
"""Next WASI 0.3 (P1) PR. Same output as wasi-p3-remaining.ps1.

W1–W8 smokes are closed (fixture exists, no remaining needle). After W8 the
script prints the first official-shape gap knife (P1-FS1 … P1-HT1). Table:
docs/mapping/gap-wasi-p3-wit.md. When those are empty, prints Next: none plus
Named-only leftovers. Do not auto-cut Defer/Out rows.
"""
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
    # After W8: official-shape gap knives. Table: docs/mapping/gap-wasi-p3-wit.md
    (
        "P1-FS1",
        "fs-preopen-list",
        "wasi:filesystem get-directories list<tuple<descriptor, string>>",
        WASI / "filesystem_preopen.wat",
        "gap: get-directories not list tuple",
        None,
    ),
    (
        "P1-FS2",
        "fs-rw-offset",
        "wasi:filesystem read/write-via-stream filesize offset",
        WASI / "filesystem_preopen.wat",
        "gap: read/write no filesize offset",
        None,
    ),
    (
        "P1-FS3",
        "fs-open-at",
        "wasi:filesystem directory preopen + open-at",
        WASI / "filesystem_preopen.wat",
        "gap: no open-at",
        None,
    ),
    (
        "P1-FS4",
        "fs-open-at-access",
        "wasi:filesystem open-at .. -> access",
        WASI / "filesystem_preopen.wat",
        "gap: open-at access not guest-visible",
        None,
    ),
    (
        "P1-SK1",
        "sockets-create-family",
        "wasi:sockets create-tcp-socket address-family result",
        WASI / "sockets_tcp.wat",
        "gap: create-tcp-socket no address-family",
        None,
    ),
    (
        "P1-SK2",
        "sockets-connect-addr",
        "wasi:sockets connect ip-socket-address result",
        WASI / "sockets_tcp.wat",
        "gap: connect no ip-socket-address",
        None,
    ),
    (
        "P1-HT1",
        "http-handle-result",
        "wasi:http incoming-handler handle result<response>",
        WASI / "http_handler.wat",
        "gap: handle not result<response>",
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
        if lane_id.startswith("P1-"):
            print("Gap: docs/mapping/gap-wasi-p3-wit.md")
    else:
        print("Next: (W1–W8 empty; official-shape gap empty)")
        print("Gap: docs/mapping/gap-wasi-p3-wit.md")
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
