# GPU Host：vendor 路径（Dawn）

[English](blocked-gpu-host.md) | **中文**

**已落地（2026-08-17）：** mvp 仓的 **Kotlin Host 映射** 已 vendor 进 `:host-dawn`。Dawn **原生库不进 git**，用已发布的 `androidx.webgpu:webgpu:1.0.0-alpha05`。钉死值在 `gradle/libs.versions.toml` → `webgpu`。

对外仍是 `:host-dawn` / `:android-webgpu` / `WebGpuBackend`。`WasiWebGpuHost` 是实现细节（首拷保留 `…experimental…` 包名）。

## 树内布局

- `experimental/host/`：`WasiWebGpuHost` / CPU / descriptor
- `experimental/dawn/DawnWasiWebGpuHost.kt`
- `experimental/abicm/`：`AbiCmHostBindings`
- `host-dawn/third_party/wasi-webgpu-jvm-mvp/`：MIT 许可与来源 commit

`:host-dawn` 直接 `api(androidx.webgpu)`。**不再** `mavenLocal()`，也**不再**依赖未发布 `experimental:*` 坐标。

**不要：** 新开 Dawn 产品仓；发布 `experimental:*` 当本仓契约；把 `.so` 提交进 git；搬 mvp 整树（4j / cube / abi-mvp）；在外仓实现 `WebGpuBackend`。

细节以英文为准。
