# GPU Host：vendor 路径（Dawn）

[English](blocked-gpu-host.md) | **中文**

**已落地（2026-08-17）：** mvp 仓的 **Kotlin Host 映射** 已 vendor 进 `:host-dawn`。Dawn **原生库不进 git**，用已发布的 `androidx.webgpu:webgpu:1.0.0-alpha05`。钉死值在 `gradle/libs.versions.toml` → `webgpu`。

**ND-DEFAULT（2026-09-01）：** 产品 `GpuBackends.dawn()` / `:android-webgpu` 走 **NativeGpu**。`DawnWasiWebGpuHost.kt` 是映射与 `id = "dawn-jni"` 剩路。默认 APK **排除** `libwebgpu_c_bundled.so`，配方产出的 `libwebgpu_dawn.so` 在构建后打进包（不要两份一起出）。

**ND-SO 钉：** 与 `androidx.webgpu:webgpu:1.0.0-alpha05` AAR 同一 Dawn SHA（`9d41fdf36977cca92361c6ae2769129bbaaafd9b`）。配方 [`../scripts/build-dawn-c-android.py`](../scripts/build-dawn-c-android.py)。`.so` 不进 git。

对外仍是 `:host-dawn` / `:android-webgpu` / `WebGpuBackend`。Maven 坐标（`0.1.0`，已发布）：`runtime` / `host-dawn` / **`android-webgpu`**。`WasiWebGpuHost` 是实现细节（首拷保留 `…experimental…` 包名）。

## 树内布局

- `experimental/host/`：`WasiWebGpuHost` / CPU / descriptor
- `experimental/dawn/DawnWasiWebGpuHost.kt`
- `experimental/abicm/`：`AbiCmHostBindings`
- `host-dawn/third_party/wasi-webgpu-jvm-mvp/`：MIT 许可与来源 commit

`:host-dawn` 直接 `api(androidx.webgpu)`。**不再** `mavenLocal()`，也**不再**依赖未发布 `experimental:*` 坐标。

**不要：** 新开 Dawn 产品仓；发布 `experimental:*` 当本仓契约；把 `.so` 提交进 git；搬 mvp 整树（4j / cube / abi-mvp）；在外仓实现 `WebGpuBackend`。

细节以英文为准。
