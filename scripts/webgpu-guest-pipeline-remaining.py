#!/usr/bin/env python3
"""Closed guest-pipeline queue. Redirects to webgpu-guest-dawn-remaining.py."""
from __future__ import annotations

import argparse


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    print("Playbook closed: docs/agent/webgpu-guest-pipeline.md")
    print("Use: python ./scripts/webgpu-guest-dawn-remaining.py")
    print("Playbook: docs/agent/webgpu-guest-pipeline.md")
    print("Next: (P1–P5 empty)")
    if args.all:
        print()
        print("=== (none) ===")


if __name__ == "__main__":
    main()
