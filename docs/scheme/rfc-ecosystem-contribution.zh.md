# RFC：生态贡献成功标准（降级原 L4）

**状态：Accepted** · 2026-08-17  
[English](rfc-ecosystem-contribution.md) | **中文**

> 修订 [`long-term-plan.md`](long-term-plan.md)。**不**改变 P0（钉版 `wasi:webgpu` WIT + 真 CM async）。  
> 若与英文冲突，以英文为准。

## 1. 决策

| 问题 | 决定 |
|------|------|
| 中期成功是什么？ | **可引用的 Android Host**：外人能复现 Guest → 本运行时 → GPU（或文档中的替代），并能据此向 [`wasi-webgpu`](https://github.com/WebAssembly/wasi-webgpu) / Wasmtime **开上游 issue**。 |
| P0 改吗？ | **不改。** |
| 原 L4（给另一 Demo 换默认 runtime）？ | **移出目标堆叠**，不再作为成功标准。 |
| Maven Central / 生产级？ | 近端仍 **不做**。可引用 ≠ 上 Central。 |
| `wasi-gfx`？ | 仍非 P0。 |

## 2. 新堆叠

```text
L0 底座 → L1 P3 原语 → L2 P3 子集 → L3 wasi:webgpu 规范形状（P0）
L4 可引用 Host（本 RFC）：复现 + 引用 + 上游笔记
L5 产品化 RFC（另开）
```

## 3. 代码依赖（未在本 RFC 解除）

仪器与 `ExperimentalWebGpuBridge` 在 vendor PR 前仍走 `:host-dawn` 的 mavenLocal。已拍板：拷 Host Kotlin；Dawn `.so` 用 `androidx.webgpu`。见 [`../blocked-gpu-host.md`](../blocked-gpu-host.md)。
