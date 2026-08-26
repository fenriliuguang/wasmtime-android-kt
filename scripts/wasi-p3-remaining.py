#!/usr/bin/env python3
"""Closed WASI 0.3 (P1) queue. Redirects to wasmtime-p2-remaining.py."""
from __future__ import annotations

import argparse


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    print("Playbook closed: docs/archive/p1-wasi-p3-playbook.md")
    print("Use: python ./scripts/wasmtime-p2-remaining.py")
    print("Playbook: docs/agent/wasmtime-p2.md")
    print("Next: (W1–W8 empty; official-shape gap empty; P1 closed)")
    if args.all:
        print()
        print("=== (none) ===")


if __name__ == "__main__":
    main()
