# RFC：生态贡献成功标准（降级原 L4）

**状态：Accepted** · 2026-08-17 · **2026-08-21 修订：** 禁止向上游开 GitHub issue。 · **2026-08-26 修订：** L5 已接受。
[English](rfc-ecosystem-contribution.md) | **中文**

> 修订 [`long-term-plan.md`](long-term-plan.md)。**不**改变 P0（钉版 `wasi:webgpu` WIT + 真 CM async）。  
> 若与英文冲突，以英文为准。

## 1. 决策

| 问题 | 决定 |
|------|------|
| 中期成功是什么？ | **可引用的 Android Host**：外人能复现 Guest → 本运行时 → GPU（或文档中的替代）。Android 事实只写本仓。**禁止**向 [`wasi-webgpu`](https://github.com/WebAssembly/wasi-webgpu) / Wasmtime 或任何上游开 GitHub issue。 |
| P0 改吗？ | **不改。** |
| 原 L4（给另一 Demo 换默认 runtime）？ | **移出目标堆叠**，不再作为成功标准。 |
| Maven Central / 生产级？ | 可引用 ≠ 上 Central。**L5：等到 `0.1.0` 门禁再发**（NG-6）。CTS / WASI 1.0 发行版仍禁止（NG-5）。 |
| `wasi-gfx`？ | 仍非 P0。最小帧循环是 **`0.1.0` 门禁**（gfx RFC）。 |

## 2. 新堆叠

```text
L0 底座 → L1 P3 原语 → L2 P3 子集 → L3 wasi:webgpu 规范形状（P0）
L4 可引用 Host（本 RFC）：复现 + 引用 + 本仓笔记
L5 产品化（已接受：长期 0.x；0.1.0 才 Central）
```

## 3. 代码依赖

仪器与 `ExperimentalWebGpuBridge` 走仓内 `:host-dawn`。Dawn `.so` 用 `androidx.webgpu`。见 [`../blocked-gpu-host.md`](../blocked-gpu-host.md)。
