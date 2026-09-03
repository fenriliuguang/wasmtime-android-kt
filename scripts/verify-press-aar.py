#!/usr/bin/env python3
"""Fail if published AARs do not contain this checkout's recipe .so files.

In-tree press gate. Does **not** clone or assemble wasmtime-android-kt-examples.
includeBuild / out-of-tree cube stays `verify-examples-gate.ps1` (local-dev).

  python3 ./scripts/verify-press-aar.py
  python3 ./scripts/verify-press-aar.py --assemble

Requires arm64 `libwasmtime_android_kt.so` and `--prebuilt` `libwebgpu_dawn.so`.
"""
from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PREBUILT_SHA = "bddf1a04f7c262107a9aae301c45fc49e15c7fef"
WASMTIME_SO = ROOT / "android" / "jniLibs" / "arm64-v8a" / "libwasmtime_android_kt.so"
DAWN_SO = (
    ROOT
    / "native"
    / "third_party"
    / "dawn-c"
    / "out"
    / "arm64-v8a"
    / "libwebgpu_dawn.so"
)
ORIGIN_PREBUILT = DAWN_SO.parent / "ORIGIN-PREBUILT.txt"


def die(msg: str, code: int = 1) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(code)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def sha256_zip_entry(aar: Path, inner: str) -> str:
    with zipfile.ZipFile(aar) as z:
        try:
            data = z.read(inner)
        except KeyError:
            names = [n for n in z.namelist() if n.endswith(".so")]
            die(f"{aar.name} missing {inner} (so entries: {names or 'none'})")
    return hashlib.sha256(data).hexdigest()


def find_release_aar(module: str) -> Path:
    out = ROOT / module / "build" / "outputs" / "aar"
    hits = sorted(out.glob("*-release.aar"))
    if not hits:
        die(
            f"missing {out}/*-release.aar — "
            "run ./gradlew :android:assembleRelease :host-dawn:assembleRelease "
            "or pass --assemble"
        )
    if len(hits) > 1:
        die(f"multiple release AARs under {out}: {hits}")
    return hits[0]


def gradlew() -> Path:
    bat = ROOT / "gradlew.bat"
    unix = ROOT / "gradlew"
    if sys.platform == "win32" and bat.is_file():
        return bat
    if unix.is_file():
        return unix
    die("missing gradlew")
    raise SystemExit(1)


def assemble() -> None:
    cmd = [
        str(gradlew()),
        ":android:assembleRelease",
        ":host-dawn:assembleRelease",
        "--no-daemon",
        "--no-configuration-cache",
    ]
    print(" ".join(cmd))
    subprocess.check_call(cmd, cwd=ROOT)


def expect_match(label: str, want: str, got: str, aar: Path, inner: str) -> None:
    if want != got:
        die(
            f"{label}: {aar.name} {inner} SHA256 {got} != recipe {want} "
            "(AAR is not the file on disk — do not press)"
        )
    print(f"ok {label} {got}")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--assemble",
        action="store_true",
        help="assemble :android and :host-dawn release AARs first",
    )
    args = ap.parse_args()
    if not WASMTIME_SO.is_file():
        die(f"missing {WASMTIME_SO} — run scripts/build-native-android.ps1")
    if not DAWN_SO.is_file():
        die(f"missing {DAWN_SO} — run scripts/build-dawn-c-android.py --prebuilt")
    if not ORIGIN_PREBUILT.is_file():
        die(
            f"missing {ORIGIN_PREBUILT} — press pin is --prebuilt, not --build "
            f"(expected SHA {PREBUILT_SHA[:12]})"
        )
    origin = ORIGIN_PREBUILT.read_text(encoding="utf-8")
    if PREBUILT_SHA not in origin:
        die(f"{ORIGIN_PREBUILT} does not name press SHA {PREBUILT_SHA}")
    if args.assemble:
        assemble()
    runtime_aar = find_release_aar("android")
    dawn_aar = find_release_aar("host-dawn")
    expect_match(
        "libwasmtime_android_kt.so",
        sha256_file(WASMTIME_SO),
        sha256_zip_entry(runtime_aar, "jni/arm64-v8a/libwasmtime_android_kt.so"),
        runtime_aar,
        "jni/arm64-v8a/libwasmtime_android_kt.so",
    )
    expect_match(
        "libwebgpu_dawn.so",
        sha256_file(DAWN_SO),
        sha256_zip_entry(dawn_aar, "jni/arm64-v8a/libwebgpu_dawn.so"),
        dawn_aar,
        "jni/arm64-v8a/libwebgpu_dawn.so",
    )
    print("press AAR .so matches recipe (in-tree; no examples repo)")


if __name__ == "__main__":
    main()
