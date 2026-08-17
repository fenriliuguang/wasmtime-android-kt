# 技术栈

[English](tech-stack.md) | **中文**

引擎：官方 `wasmtime` 47.x。绑定：Rust cdylib + JNI。Android-first。GPU：核心不含 Dawn；默认 bundle 含 Dawn。见 [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md)。现状接线见 [`../blocked-gpu-host.md`](../blocked-gpu-host.md)。
