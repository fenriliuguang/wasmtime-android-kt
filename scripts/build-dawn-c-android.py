#!/usr/bin/env python3
"""Android Dawn C API `.so` recipe (ND-SO).

Pin matches androidx.webgpu 1.0.0-alpha05 AAR dawn_build_metadata.json.
Does **not** git-add any `.so`. Does **not** enable the product default
(ND-DEFAULT). One Dawn renderer: C API adapter, not wgpu-native.

  python3 ./scripts/build-dawn-c-android.py --probe-aar
  python3 ./scripts/build-dawn-c-android.py --build [--targets arm64-v8a]

`--probe-aar` works without NDK (Cloud). `--build` needs NDK 28.2.13676358.
"""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PIN_DIR = ROOT / "native" / "third_party" / "dawn-c"
CACHE = PIN_DIR / ".cache"
OUT = PIN_DIR / "out"
INSTALL = PIN_DIR / "install"
SRC = PIN_DIR / "src"

ANDROIDX_WEBGPU = "1.0.0-alpha05"
DAWN_COMMIT = "9d41fdf36977cca92361c6ae2769129bbaaafd9b"
NDK_VERSION = "28.2.13676358"
API_LEVEL = 24
AAR_NAME = f"webgpu-{ANDROIDX_WEBGPU}.aar"
AAR_URL = (
    "https://dl.google.com/dl/android/maven2/androidx/webgpu/webgpu/"
    f"{ANDROIDX_WEBGPU}/{AAR_NAME}"
)
DAWN_REMOTES = (
    "https://dawn.googlesource.com/dawn",
    "https://github.com/google/dawn.git",
)
C_API_NEEDLES = (
    "wgpuDeviceCreateBuffer",
    "wgpuQueueOnSubmittedWorkDone",
    "wgpuInstanceRequestAdapter",
)
JNI_PREFIX = "Java_androidx_webgpu_"
BUNDLED_SO = "libwebgpu_c_bundled.so"
C_API_SO = "libwebgpu_dawn.so"


def die(msg: str, code: int = 1) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(code)


def download(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.is_file() and dest.stat().st_size > 0:
        return
    tmp = dest.with_suffix(dest.suffix + ".part")
    last_err: Exception | None = None
    for attempt in range(4):
        try:
            with urllib.request.urlopen(url, timeout=60) as resp, tmp.open("wb") as out:
                shutil.copyfileobj(resp, out)
            tmp.replace(dest)
            return
        except Exception as exc:  # noqa: BLE001 — retry network
            last_err = exc
            if tmp.exists():
                tmp.unlink()
            if attempt < 3:
                import time

                time.sleep(4 * (2**attempt))
    die(f"download failed {url}: {last_err}")


def nm_defined(path: Path) -> list[str]:
    for tool in ("nm", "llvm-nm"):
        exe = shutil.which(tool)
        if not exe:
            continue
        proc = subprocess.run(
            [exe, "-D", "--defined-only", str(path)],
            check=False,
            capture_output=True,
            text=True,
        )
        if proc.returncode == 0:
            return proc.stdout.splitlines()
    die("nm/llvm-nm not found (needed to classify JNI vs webgpu.h exports)")
    return []


def symbol_names(lines: list[str]) -> list[str]:
    names: list[str] = []
    for line in lines:
        parts = line.split()
        if len(parts) < 3:
            continue
        # addr type name  or  addr type name@@VERS
        name = parts[-1].split("@@", 1)[0]
        names.append(name)
    return names


def probe_aar() -> None:
    aar = CACHE / AAR_NAME
    print(f"fetch {AAR_URL}")
    download(AAR_URL, aar)
    print(f"aar bytes={aar.stat().st_size} path={aar} (gitignored cache)")
    with zipfile.ZipFile(aar) as zf:
        meta = zf.read("assets/dawn_build_metadata.json").decode("utf-8")
        print(f"aar dawn_build_metadata.json: {meta.strip()}")
        if DAWN_COMMIT not in meta:
            die(
                f"AAR Dawn SHA does not match pin {DAWN_COMMIT}; "
                "update ORIGIN.txt + this script together"
            )
        so_infos = []
        for info in zf.infolist():
            if info.filename.endswith("/" + BUNDLED_SO):
                so_infos.append(info)
        if not so_infos:
            die(f"{BUNDLED_SO} missing from AAR")
        arm64 = None
        for info in sorted(so_infos, key=lambda i: i.filename):
            abi = info.filename.split("/")[-2]
            mib = info.file_size / (1024 * 1024)
            print(f"  jni/{abi}/{BUNDLED_SO}  {info.file_size} bytes ({mib:.2f} MiB)")
            if abi == "arm64-v8a":
                arm64 = info
        if arm64 is None:
            die("arm64-v8a bundled .so missing")
        with tempfile.TemporaryDirectory() as td:
            extracted = Path(td) / BUNDLED_SO
            extracted.write_bytes(zf.read(arm64.filename))
            names = symbol_names(nm_defined(extracted))
            jni = [n for n in names if n.startswith(JNI_PREFIX)]
            wgpu = [n for n in names if n.startswith("wgpu")]
            print(f"arm64 defined JNI Java_androidx_webgpu_* : {len(jni)}")
            print(f"arm64 defined wgpu* (webgpu.h C API)      : {len(wgpu)}")
            for needle in C_API_NEEDLES:
                hit = any(n == needle for n in names)
                print(f"  {needle}: {'yes' if hit else 'no'}")
            if wgpu:
                die(
                    "unexpected webgpu.h exports on androidx bundled .so; "
                    "JNI leftover assumption changed"
                )
            if not jni:
                die("androidx bundled .so has no Java_androidx_webgpu_* exports")
    print(
        "probe: androidx leftover is JNI-only. NativeGpu needs a Dawn "
        f"C API build ({C_API_SO}) from pin {DAWN_COMMIT}."
    )
    print("Do not git-add the AAR or libwebgpu_c_bundled.so.")


def find_ndk() -> Path:
    env = os.environ.get("ANDROID_NDK_HOME") or os.environ.get("ANDROID_NDK_ROOT")
    if env:
        p = Path(env)
        if (p / "build/cmake/android.toolchain.cmake").is_file():
            return p
    sdk = os.environ.get("ANDROID_SDK_ROOT") or os.environ.get("ANDROID_HOME")
    candidates: list[Path] = []
    if sdk:
        candidates.append(Path(sdk) / "ndk" / NDK_VERSION)
    local = ROOT / "local.properties"
    if local.is_file():
        for line in local.read_text(encoding="utf-8").splitlines():
            if line.startswith("sdk.dir="):
                sdk_dir = line.split("=", 1)[1].strip().replace("\\", "/")
                candidates.append(Path(sdk_dir) / "ndk" / NDK_VERSION)
    home = Path.home()
    candidates.append(home / "Android" / "Sdk" / "ndk" / NDK_VERSION)
    for p in candidates:
        if (p / "build/cmake/android.toolchain.cmake").is_file():
            return p
    die(
        f"Android NDK not found (need {NDK_VERSION}). Install with: "
        f'sdkmanager --install "ndk;{NDK_VERSION}". '
        "Cloud images often have no NDK; --probe-aar still records size."
    )
    raise AssertionError


def run(cmd: list[str], cwd: Path | None = None) -> None:
    print("+", " ".join(cmd))
    subprocess.run(cmd, cwd=cwd, check=True)


def ensure_dawn_src() -> Path:
    if not (SRC / "CMakeLists.txt").is_file():
        if SRC.exists():
            shutil.rmtree(SRC)
        SRC.mkdir(parents=True)
        last: Exception | None = None
        for remote in DAWN_REMOTES:
            try:
                run(["git", "init"], cwd=SRC)
                run(["git", "remote", "add", "origin", remote], cwd=SRC)
                run(
                    ["git", "fetch", "--depth", "1", "origin", DAWN_COMMIT],
                    cwd=SRC,
                )
                run(["git", "checkout", "--force", "FETCH_HEAD"], cwd=SRC)
                last = None
                break
            except Exception as exc:  # noqa: BLE001
                last = exc
                shutil.rmtree(SRC, ignore_errors=True)
                SRC.mkdir(parents=True)
        if last is not None:
            die(f"git fetch Dawn {DAWN_COMMIT} failed: {last}")
    head = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=SRC, text=True
    ).strip()
    if not head.startswith(DAWN_COMMIT[:12]) and head != DAWN_COMMIT:
        # shallow fetch may yield full SHA
        if head != DAWN_COMMIT:
            die(f"Dawn checkout HEAD {head} != pin {DAWN_COMMIT}")
    return SRC


def build_abi(ndk: Path, abi: str) -> Path:
    src = ensure_dawn_src()
    build_dir = SRC / "out" / f"android-{abi}"
    prefix = INSTALL / abi
    toolchain = ndk / "build/cmake/android.toolchain.cmake"
    build_dir.mkdir(parents=True, exist_ok=True)
    cmake = [
        "cmake",
        "-S",
        str(src),
        "-B",
        str(build_dir),
        f"-DCMAKE_TOOLCHAIN_FILE={toolchain}",
        f"-DANDROID_ABI={abi}",
        f"-DANDROID_PLATFORM=android-{API_LEVEL}",
        "-DCMAKE_BUILD_TYPE=Release",
        "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
        "-DBUILD_SHARED_LIBS=OFF",
        "-DDAWN_BUILD_MONOLITHIC_LIBRARY=SHARED",
        "-DDAWN_FETCH_DEPENDENCIES=ON",
        "-DDAWN_ENABLE_INSTALL=ON",
        "-DDAWN_ENABLE_PIC=ON",
        "-DDAWN_USE_GLFW=OFF",
        "-DDAWN_BUILD_SAMPLES=OFF",
        "-DDAWN_BUILD_TESTS=OFF",
        "-DDAWN_ENABLE_OPENGLES=OFF",
        "-DDAWN_ENABLE_DESKTOP_GL=OFF",
        "-DDAWN_ENABLE_VULKAN=ON",
        f"-DCMAKE_INSTALL_PREFIX={prefix}",
    ]
    run(cmake)
    jobs = str(os.cpu_count() or 2)
    run(["cmake", "--build", str(build_dir), "-j", jobs, "--target", "webgpu_dawn"])
    run(["cmake", "--install", str(build_dir)])
    found: list[Path] = list(prefix.rglob(C_API_SO))
    if not found:
        found = [
            p
            for p in prefix.rglob("*.so")
            if p.name.startswith("libwebgpu") or p.name.startswith("libdawn")
        ]
    if not found:
        die(f"install prefix {prefix} has no Dawn shared library")
    so = found[0]
    dest_dir = OUT / abi
    dest_dir.mkdir(parents=True, exist_ok=True)
    dest = dest_dir / C_API_SO
    shutil.copy2(so, dest)
    names = symbol_names(nm_defined(dest))
    missing = [n for n in C_API_NEEDLES if n not in names]
    if missing:
        die(f"{dest} missing C API exports {missing}")
    mib = dest.stat().st_size / (1024 * 1024)
    print(f"built {dest}  {dest.stat().st_size} bytes ({mib:.2f} MiB)")
    print("webgpu.h install tree:", prefix / "include")
    print("Do not git-add this .so. Do not pack it next to androidx bundled in the default APK.")
    return dest


def build(targets: list[str]) -> None:
    ndk = find_ndk()
    print(f"NDK {ndk}")
    for abi in targets:
        build_abi(ndk, abi)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--probe-aar", action="store_true")
    ap.add_argument("--build", action="store_true")
    ap.add_argument(
        "--targets",
        nargs="+",
        default=["arm64-v8a"],
        help="Android ABIs for --build (default: arm64-v8a)",
    )
    args = ap.parse_args()
    if not args.probe_aar and not args.build:
        ap.print_help()
        print()
        die("pass --probe-aar and/or --build")
    print(f"Playbook: docs/agent/native-dawn.md  ND-SO")
    print(f"Dawn pin: {DAWN_COMMIT}  androidx.webgpu:{ANDROIDX_WEBGPU}")
    print(f"Output (gitignored): {OUT}/<abi>/{C_API_SO}")
    if args.probe_aar:
        probe_aar()
    if args.build:
        build(args.targets)


if __name__ == "__main__":
    main()
