#!/usr/bin/env python3
"""Closed Dawn-consume / WG-6 queue. Redirects to wasi-p3-remaining.py."""
from __future__ import annotations

import argparse


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    print("Playbook closed: docs/archive/webgpu-guest-dawn.md")
    print("Use: python ./scripts/wasi-p3-remaining.py")
    print("Playbook: docs/agent/wasi-p3.md")
    print("Next: (G1–G9 empty; P0 closed)")
    if args.all:
        print()
        print("=== (none) ===")


if __name__ == "__main__":
    main()
