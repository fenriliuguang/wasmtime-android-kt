# 技术栈

[English](tech-stack.md) | **中文**

与英文冲突时以英文为准。引擎：官方 `wasmtime` 47.x。绑定：Rust cdylib + JNI。默认消费 Dawn C；**0.1.1** 起 Maven `host-dawn` 打进 `libwebgpu_dawn.so`。见 [`../blocked-gpu-host.md`](../blocked-gpu-host.md)。
