# 技术栈

[English](tech-stack.md) | **中文**

引擎：官方 `wasmtime` 47.x。绑定：Rust cdylib + JNI。Android-first。GPU：核心不含 Dawn；默认 bundle `:android-webgpu`。Vendor：Host Kotlin 进仓；今日 Dawn `.so` 用 `androidx.webgpu`。现行改写：Dawn C 默认（[`../agent/native-dawn.md`](../agent/native-dawn.md)）。见 [`../blocked-gpu-host.md`](../blocked-gpu-host.md)。
