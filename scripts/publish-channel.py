#!/usr/bin/env python3
"""Classify a press as snapshot vs release from gradle.properties.

Publish.yml forks the Actions graph on ``channel`` so a SNAPSHOT GAV and a
non-SNAPSHOT GAV are two visible lines. This script does not upload.

  python3 ./scripts/publish-channel.py
  python3 ./scripts/publish-channel.py --tag v0.1.3-SNAPSHOT
  python3 ./scripts/publish-channel.py --event workflow_dispatch --destination github-packages
  python3 ./scripts/publish-channel.py --self-test
"""
from __future__ import annotations

import argparse
import io
import sys
import tempfile
from contextlib import redirect_stderr
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PROPERTIES = ROOT / "gradle.properties"
VERSION_KEY = "wasmtime.android.version"


def die(msg: str, code: int = 1) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(code)


def read_version(properties: Path) -> str:
    if not properties.is_file():
        die(f"missing {properties}")
    version = ""
    for raw in properties.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key.strip() == VERSION_KEY:
            version = value.strip()
    if not version:
        die(f"{properties} has no {VERSION_KEY}")
    if version.startswith("v"):
        die(f"{VERSION_KEY} must not include a v prefix ({version})")
    return version


def classify(version: str) -> str:
    if version.endswith("-SNAPSHOT"):
        stem = version[: -len("-SNAPSHOT")]
        if not stem or stem.endswith("-"):
            die(f"invalid SNAPSHOT GAV {version}")
        return "snapshot"
    if "SNAPSHOT" in version:
        die(f"GAV {version} contains SNAPSHOT but is not a -SNAPSHOT suffix")
    return "release"


def parse_destination(event: str, destination: str) -> tuple[str, bool, bool]:
    dest = "both" if event == "push" else destination
    github = False
    central = False
    if dest == "github-packages":
        github = True
    elif dest == "maven-central":
        central = True
    elif dest == "both":
        github = True
        central = True
    else:
        die(f"unknown destination: {dest}")
    return dest, github, central


def check_tag(version: str, tag: str) -> None:
    if not tag:
        return
    expected = f"v{version}"
    if tag != expected:
        die(f"tag {tag} does not match {VERSION_KEY} {version} (expected {expected})")


def emit(outputs: dict[str, str], github_output: Path | None) -> None:
    lines = [f"{key}={value}" for key, value in outputs.items()]
    text = "\n".join(lines) + "\n"
    sys.stdout.write(text)
    if github_output is not None:
        with github_output.open("a", encoding="utf-8") as fh:
            fh.write(text)


def classify_press(
    *,
    properties: Path,
    event: str,
    destination: str,
    tag: str,
) -> dict[str, str]:
    version = read_version(properties)
    channel = classify(version)
    check_tag(version, tag)
    dest, github, central = parse_destination(event, destination)
    return {
        "version": version,
        "channel": channel,
        "destination": dest,
        "publish_github": "true" if github else "false",
        "publish_central": "true" if central else "false",
    }


def write_props(path: Path, version: str) -> None:
    path.write_text(f"{VERSION_KEY}={version}\n", encoding="utf-8")


def self_test() -> None:
    failures: list[str] = []

    def expect_ok(label: str, version: str, **kwargs: object) -> dict[str, str] | None:
        with tempfile.TemporaryDirectory() as tmp:
            props = Path(tmp) / "gradle.properties"
            write_props(props, version)
            try:
                with redirect_stderr(io.StringIO()):
                    return classify_press(properties=props, **kwargs)  # type: ignore[arg-type]
            except SystemExit as exc:
                failures.append(f"{label}: expected ok, got exit {exc.code}")
                return None

    def expect_fail(label: str, version: str, **kwargs: object) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            props = Path(tmp) / "gradle.properties"
            write_props(props, version)
            try:
                with redirect_stderr(io.StringIO()):
                    classify_press(properties=props, **kwargs)  # type: ignore[arg-type]
            except SystemExit:
                return
            failures.append(f"{label}: expected failure")

    got = expect_ok(
        "snapshot GAV",
        "0.1.3-SNAPSHOT",
        event="push",
        destination="github-packages",
        tag="v0.1.3-SNAPSHOT",
    )
    if got != {
        "version": "0.1.3-SNAPSHOT",
        "channel": "snapshot",
        "destination": "both",
        "publish_github": "true",
        "publish_central": "true",
    }:
        failures.append(f"snapshot GAV: {got}")

    got = expect_ok(
        "release GAV",
        "0.1.2",
        event="workflow_dispatch",
        destination="maven-central",
        tag="",
    )
    if got != {
        "version": "0.1.2",
        "channel": "release",
        "destination": "maven-central",
        "publish_github": "false",
        "publish_central": "true",
    }:
        failures.append(f"release GAV: {got}")

    got = expect_ok(
        "dispatch github-packages",
        "0.1.3-SNAPSHOT",
        event="workflow_dispatch",
        destination="github-packages",
        tag="",
    )
    if got and got["publish_github"] != "true" or got and got["publish_central"] != "false":
        failures.append(f"dispatch github-packages: {got}")

    expect_fail(
        "tag mismatch",
        "0.1.3-SNAPSHOT",
        event="push",
        destination="both",
        tag="v0.1.3",
    )
    expect_fail(
        "v-prefixed version",
        "v0.1.2",
        event="push",
        destination="both",
        tag="v0.1.2",
    )
    expect_fail(
        "bad SNAPSHOT shape",
        "-SNAPSHOT",
        event="push",
        destination="both",
        tag="v-SNAPSHOT",
    )
    expect_fail(
        "embedded SNAPSHOT",
        "0.1.3-SNAPSHOT-1",
        event="workflow_dispatch",
        destination="both",
        tag="",
    )
    expect_fail(
        "unknown dest",
        "0.1.2",
        event="workflow_dispatch",
        destination="npm",
        tag="",
    )

    with tempfile.TemporaryDirectory() as tmp:
        props = Path(tmp) / "gradle.properties"
        props.write_text("wasmtime.android.group=x\n", encoding="utf-8")
        try:
            with redirect_stderr(io.StringIO()):
                classify_press(
                    properties=props,
                    event="push",
                    destination="both",
                    tag="",
                )
            failures.append("missing version: expected failure")
        except SystemExit:
            pass

    live = classify_press(
        properties=DEFAULT_PROPERTIES,
        event="workflow_dispatch",
        destination="both",
        tag="",
    )
    if got_live_mismatch := (
        live["version"] != read_version(DEFAULT_PROPERTIES)
        or live["channel"] not in {"snapshot", "release"}
    ):
        failures.append(f"live gradle.properties: {live} mismatch={got_live_mismatch}")

    if failures:
        for item in failures:
            print(f"FAIL {item}", file=sys.stderr)
        die(f"{len(failures)} self-test failure(s)")
    print("ok publish-channel self-test")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--properties", type=Path, default=DEFAULT_PROPERTIES)
    ap.add_argument("--event", choices=("push", "workflow_dispatch"), default="workflow_dispatch")
    ap.add_argument(
        "--destination",
        default="both",
        help="workflow_dispatch destination; ignored for tag push (always both)",
    )
    ap.add_argument("--tag", default="", help="GITHUB_REF_NAME for tag pushes; empty on dispatch")
    ap.add_argument("--github-output", type=Path, default=None)
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        self_test()
        return
    outputs = classify_press(
        properties=args.properties,
        event=args.event,
        destination=args.destination,
        tag=args.tag,
    )
    emit(outputs, args.github_output)
    print(
        f"press line {outputs['channel']} GAV {outputs['version']} dest {outputs['destination']}",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
