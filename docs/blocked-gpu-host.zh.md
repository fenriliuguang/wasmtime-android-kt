# GPU Host：vendor 路径（Dawn）

[English](blocked-gpu-host.md) | **中文**

**已拍板（2026-08-17）：** 把 mvp 仓的 **Kotlin Host 映射** vendor 进本仓。Dawn **原生库不进 git**，继续用已发布的 `androidx.webgpu:webgpu`。

在 copy PR 落地前，`:host-dawn` 仍走 mavenLocal（仅 GPU 构建的停车标志）。

## 形式

从 `wasi-webgpu-jvm-mvp` 拷入（`:host-dawn-impl` 或并入 `:host-dawn`）：

- `host-api`（`WasiWebGpuHost` / CPU / descriptor）
- `DawnWasiWebGpuHost`
- `AbiCmHostBindings`

Dawn `.so` 来自 Google Maven：`androidx.webgpu:webgpu:1.0.0-alpha05`。

**不要：** 新开 Dawn 产品仓；发布 `experimental:*` 当本仓契约；把 `.so` 提交进 git；搬 mvp 整树（4j / cube / abi-mvp）；在外仓实现 `WebGpuBackend`。

对外仍是 `:host-dawn` / `:android-webgpu` / `WebGpuBackend`。细节以英文为准。
