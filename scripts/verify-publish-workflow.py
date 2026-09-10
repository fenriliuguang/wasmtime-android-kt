#!/usr/bin/env python3
"""Structural checks for the staged Publish workflow graph.

Does not talk to GitHub. Parses `.github/workflows/publish.yml` as text so
PyYAML's `on:` → True quirk cannot hide a missing trigger.

  python3 ./scripts/verify-publish-workflow.py
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github" / "workflows" / "publish.yml"


def die(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(1)


def job_block(text: str, job_id: str) -> str:
    pattern = rf"(?ms)^  {re.escape(job_id)}:\n(.*?)(?=^  [A-Za-z0-9_]+:|\Z)"
    match = re.search(pattern, text)
    if not match:
        die(f"missing job {job_id}")
    return match.group(1)


def needs_of(block: str) -> list[str]:
    match = re.search(r"needs:\s*\[([^\]]+)\]", block)
    if not match:
        die(f"missing needs: [...] in job block:\n{block[:200]}")
    return [part.strip() for part in match.group(1).split(",")]


def has_environment(block: str) -> bool:
    return bool(re.search(r"^\s+environment:", block, re.M))


def main() -> None:
    if not WORKFLOW.is_file():
        die(f"missing {WORKFLOW}")
    text = WORKFLOW.read_text(encoding="utf-8")
    failures: list[str] = []

    if "name: Publish" not in text:
        failures.append("workflow name is not Publish")
    if "environment: release" not in text and 'name: release' not in text:
        failures.append("upload job must keep GitHub Environment release")

    for job_id in ("gate", "native", "dawn", "pack", "snapshot", "release"):
        if not re.search(rf"^  {job_id}:", text, re.M):
            failures.append(f"missing job {job_id}")

    gate = job_block(text, "gate")
    native = job_block(text, "native")
    dawn = job_block(text, "dawn")
    pack = job_block(text, "pack")
    snapshot = job_block(text, "snapshot")
    release = job_block(text, "release")

    if "scripts/publish-channel.py" not in gate:
        failures.append("gate must classify via scripts/publish-channel.py")
    if has_environment(gate):
        failures.append("gate must not wait on Environment release (keep it cheap)")
    if has_environment(native):
        failures.append("native must not use Environment release (retry without re-approval)")
    if has_environment(dawn):
        failures.append("dawn must not use Environment release")
    if has_environment(pack):
        failures.append("pack must not use Environment release")
    if not has_environment(snapshot):
        failures.append("snapshot / publish must use Environment release (NG-6)")
    if not has_environment(release):
        failures.append("release / publish must use Environment release (NG-6)")

    if needs_of(native) != ["gate"]:
        failures.append(f"native needs {needs_of(native)}, expected [gate]")
    if needs_of(dawn) != ["gate"]:
        failures.append(f"dawn needs {needs_of(dawn)}, expected [gate] (parallel with native)")
    pack_needs = needs_of(pack)
    if "native" not in pack_needs or "dawn" not in pack_needs:
        failures.append(f"pack needs {pack_needs}, expected native and dawn")
    for line_name, block in (("snapshot", snapshot), ("release", release)):
        pub_needs = needs_of(block)
        if "pack" not in pub_needs or "gate" not in pub_needs:
            failures.append(f"{line_name} needs {pub_needs}, expected gate and pack")
        if "native" in pub_needs or "dawn" in pub_needs:
            failures.append(f"{line_name} must not need native/dawn directly (retry upload from pack artifacts)")

    if "needs.gate.outputs.channel == 'snapshot'" not in snapshot:
        failures.append("snapshot line must run only when channel is snapshot")
    if "needs.gate.outputs.channel == 'release'" not in release:
        failures.append("release line must run only when channel is release")
    if "press-publish" not in snapshot or "press-publish" not in release:
        failures.append("both lines must upload via press-publish")

    if "press-jniLibs" not in native or "press-jniLibs" not in pack:
        failures.append("native must upload press-jniLibs for pack/publish retry")
    if "press-dawn-c" not in dawn or "press-dawn-c" not in pack:
        failures.append("dawn must upload press-dawn-c for pack/publish retry")
    if "verify-press-aar.py" not in pack:
        failures.append("pack must run verify-press-aar.py")

    if failures:
        for item in failures:
            print(f"FAIL {item}", file=sys.stderr)
        die(f"{len(failures)} publish.yml check(s) failed")
    print("ok staged Publish graph: gate → native ∥ dawn → pack → snapshot|release")


if __name__ == "__main__":
    main()
