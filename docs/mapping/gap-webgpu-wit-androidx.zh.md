# 差距：`wasi:webgpu` WIT ↔ host ↔ androidx.webgpu

[English](gap-webgpu-wit-androidx.md) | **中文**

P0 已关闭。钉版 `wasi:webgpu@0.3.0-rc.2`；Dawn AAR `androidx.webgpu:webgpu:1.0.0-alpha05`。本页不是切刀队列。收口：[`../archive/p0-wasi-webgpu.zh.md`](../archive/p0-wasi-webgpu.zh.md)。现行：[`../agent/product-010.md`](../agent/product-010.md)。与英文冲突时以英文为准。

## 覆盖

`[method]` 已覆盖钉版 resource 方法。S 系列 / F1–F9 / G1–G9 / WG-6 guest 画出的 compute·3D·上屏已落地。未接线 `request-adapter` → guest `none`。

## androidx 空洞（只进 Kotlin record）

| WIT | 现状 |
|-----|------|
| shader `compilation-hints` | record 有；`GPUShaderModuleDescriptor` 无槽 |
| canvas `color-space` / `tone-mapping` | leftover record 有；`GPUSurfaceConfiguration` 无槽 |

blend / cull / MSAA / view-formats / xr / default-queue / write-mask / stencil / surface viewFormats·alphaMode / `layout: auto` 已在此 AAR 上消费。等 bump AAR 再拷，不要重切 G3/G8。

## 其它

`get-*` 是测试构造器。experimental 扁平面冻结。不宣称 CTS；不上 wasi-gfx；GPU 不走 `wasmtime-wasi`。

真机现象（V2458A / Mali：连续 present 抽搐升高后 GpuThread SIGSEGV；开局转得过快；**仍抖**）记在英文 §5。完整排查表：[`gfx-hitch-checklist.zh.md`](gfx-hitch-checklist.zh.md)。本分支：acquire 不再等上一帧 GPU；poller 上 retire；`writeBuffer` 加锁+复用 buffer。`on-frame` **不锁 60Hz**；pin `frame-event` 无时间戳，运动 delta 用 `monotonic-clock.now`。Guest 自有纹理不扫。
