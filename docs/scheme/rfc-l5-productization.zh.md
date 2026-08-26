# RFC：L5 产品化（上游 1.0 前长期 0.x）

**Status: Accepted** · 2026-08-26  
[English](rfc-l5-productization.md) | **中文**

英文为正文。本页为忠实摘要。

## 决定

产品类 **B**（给 App 用的 Android Component 运行时）。第一 class 是**提案中的提案**（`wasi:webgpu`，然后 `wasi-gfx`）。版本学 three.js：**长期 `0.x.y`**。本仓 **1.0.0** 前提：`wasi:webgpu` 与 `wasi-gfx` 成为正式 WASI、WASI 发 1.0、`androidx.webgpu` 发正式版。本仓 1.0 实现清单另议。

- 引擎：只钉官方 `wasmtime`（现 47.x patch）；**只**真 CM async。
- WASI：**产品子集**，不是 testsuite。「全量 0.3」= 0.1 门禁表，不是 NG-4。
- `0.1.0`：大部分钉版 `wasi:webgpu` WIT + 业务所需 IO/网络 + [gfx 帧循环 RFC](rfc-wasi-gfx-frame-loop.md)。Dawn/androidx 无槽位列为已知限制。不宣称 CTS。
- **门禁未到不发 Central / GitHub Packages**（不做 `0.0.x-preview`）。
- groupId：`io.github.fenriliuguang.wasmtime.android`。三坐标：`runtime` / `host-dawn` / **`android-webgpu`（0.x 默认）**。AAR 带 `.so`；也可自编译；Dawn 走 androidx 传递依赖。
- GPU：**双轨** — `setWebGpuBackend` 为稳定合同；ServiceLoader 为默认 bundle 便利。R8 consumer rules 随 AAR。
- `0.1.0` 前把 `ExperimentalHostCallbacks` **移出** `runtime` 公共 SPI。
- Fixture **仅测试**。
- 帧循环单开 RFC，作为 0.1 门禁，**不是**重开 P0。
