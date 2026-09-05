#!/usr/bin/env python3
"""Next WASI 0.3 leftover *commit* on cursor/wasi-p3-leftover-b677.

NG-4 stays: not wasi-testsuite / “full WASI 0.3”. Thin host only.
Do not add wasmtime-wasi. Do not open a PR until leftover is empty.
"""
from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TRACKING = ROOT / "docs" / "scheme" / "wasi-p3-leftover.md"

LANES = (
    (
        "L-ERR-CLI",
        "cli-error-code-enum",
        "CliErrorCode matches official wasi:cli error-code; one extra guest err path",
        "gap: l-err-cli pending",
    ),
    (
        "L-ERR-FS",
        "filesystem-error-code-enum",
        "FsErrorCode matches official wasi:filesystem error-code",
        "gap: l-err-fs pending",
    ),
    (
        "L-ERR-SOCK",
        "sockets-error-code-enum",
        "SockErrorCode matches official sockets error-code",
        "gap: l-err-sock pending",
    ),
    (
        "L-ERR-HTTP",
        "http-error-code-enum",
        "HttpErrorCode matches official wasi:http error-code used by send/body",
        "gap: l-err-http pending",
    ),
    (
        "L-CMD-ENV",
        "cli-environment",
        "wasi:cli/environment get-environment / get-arguments",
        "gap: l-cmd-env pending",
    ),
    (
        "L-CMD-EXIT",
        "cli-exit",
        "wasi:cli/exit; do not kill the ART process",
        "gap: l-cmd-exit pending",
    ),
    (
        "L-CMD-TERM",
        "cli-terminal",
        "terminal-stdin/stdout/stderr (Android none allowed)",
        "gap: l-cmd-term pending",
    ),
    (
        "L-FS-STAT",
        "filesystem-stat",
        "stat / stat-at on the sandbox descriptor",
        "gap: l-fs-stat pending",
    ),
    (
        "L-FS-DIR",
        "filesystem-read-directory",
        "read-directory as a CM stream",
        "gap: l-fs-dir pending",
    ),
    (
        "L-FS-APPEND",
        "filesystem-append",
        "append-via-stream",
        "gap: l-fs-append pending",
    ),
    (
        "L-FS-SYNC",
        "filesystem-sync",
        "sync / sync-data",
        "gap: l-fs-sync pending",
    ),
    (
        "L-FS-TIMES",
        "filesystem-set-times",
        "set-times / set-times-at",
        "gap: l-fs-times pending",
    ),
    (
        "L-SOCK-LISTEN",
        "sockets-tcp-listen",
        "TCP bind/listen/accept; default sandbox loopback only",
        "gap: l-sock-listen pending",
    ),
    (
        "L-SOCK-UDP",
        "sockets-udp",
        "udp-create-socket + send/receive subset",
        "gap: l-sock-udp pending",
    ),
    (
        "L-SOCK-DNS",
        "sockets-ip-name-lookup",
        "ip-name-lookup on a helper thread",
        "gap: l-sock-dns pending",
    ),
    (
        "L-HTTP-FIELDS",
        "http-fields-headers",
        "wasi:http fields/headers on request/response",
        "gap: l-http-fields pending",
    ),
    (
        "L-HTTP-TRAIL",
        "http-trailers",
        "trailers on consume-body option",
        "gap: l-http-trail pending",
    ),
    (
        "L-HTTP-TLS",
        "http-tls-https",
        "https on client.send; changelog size + Android thread; no wasmtime-wasi",
        "gap: l-http-tls pending",
    ),
    (
        "L-HTTP-SVC",
        "http-incoming-handler-shape",
        "remaining incoming-handler/types shape; not a listen HTTP server",
        "gap: l-http-svc pending",
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

    print("Playbook: docs/scheme/wasi-p3-leftover.md")
    print("Branch: cursor/wasi-p3-leftover-b677")
    if leftover:
        lane_id, title, method = leftover[0]
        print(f"Next: {lane_id} {title}")
        print(f"  {method}")
        print("Tracking: docs/scheme/wasi-p3-leftover.md")
    else:
        print("Next: (WASI 0.3 leftover empty)")
        print("Tracking: docs/scheme/wasi-p3-leftover.md")
        print(
            "Named-only: wasi-testsuite, wasmtime-wasi, 0.2 pollable, "
            "clocks timezone, stackful CM async, benches, gfx unconfigure, "
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
