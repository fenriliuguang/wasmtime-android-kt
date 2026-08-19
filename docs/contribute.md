# Contributor build and desktop shell

**English** | [中文](contribute.zh.md)

How to set up locally and iterate L1 on a desktop JVM. Formal Android reproduction: [`build.md`](build.md) + [`mapping/artifacts.md`](mapping/artifacts.md).  
Branches / PRs: [`../CONTRIBUTING.md`](../CONTRIBUTING.md) and [`scheme/vcs-workflow.md`](scheme/vcs-workflow.md).

## 1. Scope

| Do | Do not |
|----|--------|
| Reproduce Android `.so` + instrument gates | Treat the desktop shell as CI / DoD |
| Optionally build host-OS native and run JVM smokes | Panama ([`scheme/non-goals.md`](scheme/non-goals.md) DG-1) |
| Add a `changelog/unreleased/` fragment for docs / fixtures / Kotlin API | Introduce wasmtime4j as the runtime (NG-2) |
| Short-lived branch + PR per [`vcs-workflow.md`](scheme/vcs-workflow.md) | Standing multi-`feature/*` forks then a big merge |

Principle: **Android-first**; desktop is convenience.

## 2. Minimum tools

| Tool | Pin / notes |
|------|-------------|
| JDK | 17+ (Gradle Daemon often 21) |
| Rust | **1.97.1** (`native/rust-toolchain.toml`) |
| cargo-ndk | Android cross-compile only |
| Android SDK + NDK `28.2.13676358` | Device / emulator instruments |
| GPU host | [`blocked-gpu-host.md`](blocked-gpu-host.md) — Host Kotlin in `:host-dawn`; Dawn `.so` via `androidx.webgpu` (not git) |

Cursor / VS Code: compile Kotlin with Gradle. Do not let Red Hat Java (JDT LS) import/autobuild Gradle — it copies `.kt` into `runtime-jni/bin/`. [`.vscode/settings.json`](../.vscode/settings.json) already turns that off.

## 3. Workflows

### 3.1 Android main path (gate)

```powershell
.\scripts\build-native-android.ps1
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

OEM instrument notes: [`build.md`](build.md).

### 3.2 Optional desktop shell (L1 iteration)

Without NDK, build the same cdylib for the host OS and smoke `loadLibrary` / version probe on desktop JVM.

```powershell
.\scripts\build-native-host.ps1
.\gradlew.bat :runtime-api:compileKotlin :runtime-jni:compileKotlin
.\gradlew.bat :runtime-jni:test
```

Manual JVM:

```powershell
java -Djava.library.path="$PWD\desktop\jniLibs" ...
```

| Item | Value |
|------|-------|
| Output | `desktop/jniLibs/` (**not** official; gitignored) |
| Windows | `wasmtime_android_kt.dll` |
| Linux / macOS | `libwasmtime_android_kt.so` / `.dylib` |
| `System.loadLibrary` | still `wasmtime_android_kt` |

Constraints:

- Do **not** write `android/jniLibs/`; do not replace the dual-ABI layout.  
- Dawn / Surface / on-screen smoke **still** go through Android instruments.  
- Missing host lib: `:runtime-jni:test` fails and tells you to run `build-native-host.ps1`.

## 4. Docs to touch with code

| Change | At least update |
|--------|-----------------|
| Toolchain / ABI pins | `docs/build.md`, `scheme/tech-stack.md`, `changelog/unreleased/` fragment |
| Public API / error types | `scheme/api-stability.md`, `mapping/errors.md`, fragment |
| GPU host | [`blocked-gpu-host.md`](blocked-gpu-host.md) — vendor Host Kotlin; Dawn via `androidx.webgpu` |
| WASI / webgpu scope | [`scheme/long-term-plan.md`](scheme/long-term-plan.md), [`wasi-p3-surface.md`](scheme/wasi-p3-surface.md), [`roadmap-wasi-webgpu.md`](scheme/roadmap-wasi-webgpu.md) |

## 5. PR summary

Full rules: [`scheme/vcs-workflow.md`](scheme/vcs-workflow.md).

1. Short-lived branch from latest `main` (`docs/` / `feat/` / `fix/`).  
2. One PR, one thing; docs with behavior; [`changelog/unreleased/`](../changelog/unreleased/README.md) fragment (**not** root `CHANGELOG.md`).  
3. Instrument green outranks desktop green; if desktop fails but Android is green, note the host OS.  
4. experimental `0.x`: breaking changes need a fragment ([`api-stability.md`](scheme/api-stability.md)).  
5. Do not churn hub files (root README index, `ci.yml` test lists, `CONTRIBUTING.md`) unless the PR **is** policy — [`CONTRIBUTING.md`](../CONTRIBUTING.md) hub freeze.

## 6. Links

- [`scheme/vcs-workflow.md`](scheme/vcs-workflow.md) — branches / PRs  
- [`scheme/long-term-plan.md`](scheme/long-term-plan.md)  
- [`build.md`](build.md)  
- [`mapping/artifacts.md`](mapping/artifacts.md)  
- [`agent/webgpu-shape-slice.md`](agent/webgpu-shape-slice.md) — S6+ `[method]` shape-hang playbook  
- [`agent/webgpu-semantic-l2.md`](agent/webgpu-semantic-l2.md) — semantic L2 (caller then JNI family, described JNI)  
- [`../archive/README.md`](archive/README.md) — historical M0–M5  
