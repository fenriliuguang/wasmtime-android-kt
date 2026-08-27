#!/usr/bin/env python3
"""Closed leftover-descriptor queue. Redirects to product-010-remaining.py."""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("-All", "--all", action="store_true")
    args = ap.parse_args()
    print("Playbook closed: docs/archive/webgpu-guest-semantics.md", flush=True)
    print("Use: python ./scripts/product-010-remaining.py", flush=True)
    script = Path(__file__).with_name("product-010-remaining.py")
    cmd = [sys.executable, str(script)]
    if args.all:
        cmd.append("--all")
    subprocess.check_call(cmd)


if __name__ == "__main__":
    main()
