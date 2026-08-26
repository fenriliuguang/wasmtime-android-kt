#!/usr/bin/env python3
"""Closed guest-pipeline queue. Redirects to wasmtime-p2-remaining.py."""
from __future__ import annotations

import argparse


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    print("Playbook closed: docs/archive/webgpu-guest-pipeline.md")
    print("Use: python ./scripts/wasmtime-p2-remaining.py")
    print("Playbook: docs/agent/wasmtime-p2.md")
    print("Next: (P1–P5 empty; P0 closed)")
    if args.all:
        print()
        print("=== (none) ===")


if __name__ == "__main__":
    main()
