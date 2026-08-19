# Agent notes

WebGPU product `[method]` **shape hangs**: follow [`docs/agent/webgpu-shape-slice.md`](docs/agent/webgpu-shape-slice.md) (Cursor skill `webgpu-shape-slice`). Remaining names: `.\scripts\webgpu-shape-remaining.ps1`.

WebGPU **semantic L2** (guest fields → JNI → `WasiWebGpuHost`): follow [`docs/agent/webgpu-semantic-l2.md`](docs/agent/webgpu-semantic-l2.md) (Cursor skill `webgpu-semantic-l2`). Remaining: `.\scripts\webgpu-semantic-l2-remaining.ps1`. Batch by caller resource, then one JNI family per PR.

- WIT pin is vendored: `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Do not download it.
- Hub freeze, narrow tests, copy sources, and file whitelist are in the playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.

## Cursor Cloud specific instructions

Linux VM. Available: Rust `1.97.1` (installed by the startup update script; auto-selected in `native/` via `native/rust-toolchain.toml`) and JDK 21. The Gradle wrapper (`gradle-9.6.1`) auto-downloads on first use.

Out of scope in this VM: Android SDK/NDK, `cargo-ndk`, an emulator/device, and the unpublished GPU host ([`docs/blocked-gpu-host.md`](docs/blocked-gpu-host.md)) are **not** present. So the Android instrument gate (`scripts/build-native-android.ps1`, `:smoke-app:connectedDebugAndroidTest`) and any Dawn/`androidx.webgpu` module (`:host-dawn`, `:android-webgpu`, `:smoke-app`) cannot build/run here. Cloud scope = native Rust + `:runtime-api` compile + the optional desktop shell.

Runnable here:
- Native tests (CI gate 1): `cd native && cargo test --locked --tests` (first run compiles wasmtime 47.x, a few minutes).
- JVM compile (CI gate 2): `sh ./gradlew :runtime-api:compileKotlin`.
- Desktop shell end-to-end (real JNI → Rust runtime): build the host cdylib, then `sh ./gradlew :runtime-jni:test`.

Non-obvious caveats:
- `gradlew` is committed **non-executable**; invoke it as `sh ./gradlew ...` (don't commit a `chmod +x` mode change).
- The `scripts/*.ps1` are PowerShell and there's no `pwsh` here. To reproduce `build-native-host.ps1`: `cd native && cargo build --release`, then copy `target/release/libwasmtime_android_kt.so` into `desktop/jniLibs/` (gitignored). `:runtime-jni:test` `assumeTrue`-skips if that `.so` is missing, so build it first or the smoke silently no-ops.
- `cargo fmt --all -- --check` reports many **pre-existing** diffs (e.g. `native/src/cm.rs`); the repo is not kept globally rustfmt-clean. Only rustfmt the `.rs` files you change (per the playbook). CI does not gate on fmt/clippy — only `cargo test`.
