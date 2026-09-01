# `0.1.0` 宣称表（不是 CTS）

[English](claim-010.md) | **中文**

L5 [`rfc-l5-productization.md`](rfc-l5-productization.md) §7–§8 的发布说明形 **产品子集**。与英文冲突时以英文为准。**不是**合规宣称（NG-4 / NG-5）。L5 子集坐标 **`0.1.0`**；默认消费 Dawn C 在 **`0.2.0`**。现行：[`../agent/native-dawn.md`](../agent/native-dawn.md)（全量钉 Dawn C；立方体只是演示）。发布 CI：[`.github/workflows/publish.yml`](../../.github/workflows/publish.yml)（secrets / arm64 `.so` 缺失时不要强发）。

## 一句话

第三方可以依赖：钉住的 **`wasi:webgpu@0.3.0-rc.2` 绝大多数 `[method]` 走 Dawn C / NativeGpu 消费**（不是「JNI instantiate」）、webgpu 应用需要的 **WASI 0.3 IO/网络子集**、以及 **完整 `wasi-gfx` 上屏循环**（产品 adapter/device + vsync）。**P010-GFXV** 已落地 Choreographer vsync（1-slot；`surfaceDestroyed` 关 stream）。**P010-DEMO** 已链仓外 wasm→运行时→上屏仓库，并在 §6 写一行真机。Record 空洞与 named-only WASI 剩项公开列出，不当静默丢字段。

## `wasi:webgpu`

钉 WIT 里 224 个 resource `[method]` 均已在 `native/src/cm.rs` 注册；默认 `GpuBackends.dawn()` 走进程内 NativeGpu（Dawn C 槽位在 `.so` 未加载前为 0）。未接线时 `request-adapter` 为 **`none`**。`dawn-jni` 是显式剩路。缺槽（shader `compilation-hints`、canvas `color-space` / `tone-mapping`）是 **Record** 不是 Dawn C：见 [`../mapping/gap-webgpu-native-dawn.md`](../mapping/gap-webgpu-native-dawn.md)。JNI 映射：[`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md)。Fixture `get-gpu` / `get-device` 不是产品表面。不是 CTS。

## WASI 产品子集 vs 点名

| 包 | `0.1.0` 产品 | 点名（非本门禁） |
|----|--------------|------------------|
| clocks / random / stream | 已落地函数 | — |
| cli | stdio + `run` + NUL → `illegal-byte-sequence` | G-err 全枚举、G-cmd |
| filesystem | 预开目录 + `open-at` + 读写；`..` → `access` | G-fs-full |
| sockets | 出站 TCP 拨非回环 IPv4 | listen / UDP / DNS |
| http | body `stream<u8>` + 线上 GET；产品无 request/response 构造器 | service 世界、TLS |
| gfx | `surface@0.2.0` + 产品 adapter/device + Choreographer vsync（P010-GFXV）；**P010-DEMO**（README 仓外 demo + 真机行） | 完整桌面 gfx |

不宣称 `wasmtime-wasi`、不宣称本仓 1.0。发布 workflow 已落地（P010-PUB）；secrets 缺失时不要强发 Central。

## 真机上屏

原生默认仪器（ND-DEVICE，`GpuBackends.dawn()`）：`WasiGfxFrameLoopInstrumentedTest`、WG-6 guest compute/render/canvas present、`WasiWebGpuMethodCanvasContextPresentInstrumentedTest`；Cpu 寿命孪生 `WasiWebGpuCanvasContextFrameLifetimeInstrumentedTest`。Cloud 无真机，仅点名。立方体是演示行，不是 consume DoD。

| 设备 | ABI | Android | 路径 | 日期 |
|------|-----|---------|------|------|
| Vivo V2458A（PD2415M） | arm64-v8a | 16 | `WasiGfxFrameLoopInstrumentedTest`（P010-GFXV vsync 上屏） | 2026-08-27 |
| Vivo V2458A（PD2415M） | arm64-v8a | 16 | 仓外旋转立方体（[wasmtime-android-kt-examples](https://github.com/fenriliuguang/wasmtime-android-kt-examples)）— 仅演示 | 2026-08-27 |

Cloud 无真机。不是 CTS。app 不 vendor 进本仓。
