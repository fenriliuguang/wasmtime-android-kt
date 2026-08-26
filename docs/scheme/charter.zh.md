# 章程

[English](charter.md) | **中文**

Android-first 的 **Java/Kotlin Component 运行时**，引擎为 **官方 Wasmtime**。

首发提案世界：规范形状的 [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu)（WIT + 真 CM async；**P0 已关闭**）。P1 WASI 0.3 **已关闭**。现行工作为 P2 Wasmtime 钉。运行时不永远绑死单一 world。

仍为 **experimental**。成功标准见 [`rfc-ecosystem-contribution.zh.md`](rfc-ecosystem-contribution.zh.md)。  
GPU：默认产品/测试带 Dawn；核心 AAR 不含 Dawn。Vendor：Host Kotlin 进仓，Dawn `.so` 用 androidx.webgpu。见 [`../blocked-gpu-host.md`](../blocked-gpu-host.md)。

与英文冲突时以英文为准。
