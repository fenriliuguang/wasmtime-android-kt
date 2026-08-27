# Native artifact layout

**English** | [中文](artifacts.zh.md)

Official location, ABIs, and verify entry for `libwasmtime_android_kt.so`.

## Official layout

Root: `android/jniLibs/` (consumed by `:android` AAR / `smoke-app`; **gitignored**).

```text
android/jniLibs/
  arm64-v8a/libwasmtime_android_kt.so   # device primary ABI (required)
  x86_64/libwasmtime_android_kt.so      # emulator / desktop Android (required for official dual ABI)
  build-info.json                       # build metadata (script-generated, gitignored)
```

| Item | Value |
|------|-------|
| `System.loadLibrary` | `wasmtime_android_kt` (`NativeLibraryNames.WASMTIME_ANDROID_KT`) |
| File name | `libwasmtime_android_kt.so` |
| min API | 24 (matches `cargo ndk --platform`) |
| NDK | `28.2.13676358` |
| Rust | `1.97.1` |

## Build

```powershell
.\scripts\build-native-android.ps1
.\scripts\build-native-android.ps1 -Targets arm64-v8a
.\scripts\verify-native-android.ps1
.\scripts\verify-native-android.ps1 -RequireAll
```

After success the script:

1. `llvm-strip --strip-unneeded` (`-SkipStrip` to skip)  
2. Writes `android/jniLibs/build-info.json`  
3. Verifies this run’s `-Targets`

## `build-info.json`

| Field | Meaning |
|-------|---------|
| `libraryFile` | `libwasmtime_android_kt.so` |
| `loadLibrary` | `wasmtime_android_kt` |
| `ndkVersion` / `apiLevel` / `rustToolchain` | pinned toolchain |
| `abis.<abi>.bytes` / `sha256` | size and content hash |
| `builtAt` | UTC ISO-8601 |

## Verify (`verify-native-android.ps1`)

- Path: `android/jniLibs/<abi>/libwasmtime_android_kt.so`  
- Exists and `bytes >= 1 MiB` (empty/truncated guard)  
- `-RequireAll`: both `arm64-v8a` and `x86_64`  
- If `build-info.json` exists, optionally check sha256 (default: listed ABIs)

## Optional desktop host artifacts (not official)

[`../contribute.md`](../contribute.md):

```powershell
.\scripts\build-native-host.ps1
# → desktop/jniLibs/wasmtime_android_kt.dll|.so|.dylib + build-info.json
```

- Does **not** enter the AAR / instrument gate; does **not** replace dual ABI above.  
- JVM: `-Djava.library.path=<repo>/desktop/jniLibs` (`:runtime-jni:test` is already configured).

## Non-goals

- AAR ships `android/jniLibs/<abi>/libwasmtime_android_kt.so` (arm64-v8a required; x86_64 emulator in 0.x)
- Publishing CI: [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml) — same GAV on GitHub Packages + Maven Central
- Do not commit `.so`
- No `armeabi-v7a` / `x86` unless a dedicated RFC  
