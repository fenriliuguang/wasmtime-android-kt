# 错误模型（L1）

[English](errors.md) | **中文**

Kotlin 异常：`WasmtimeApiException` / `Compile` / `Link` / `Trap`，基类 `WasmtimeException`。

遗留扁平 experimental Host 失败仍映射为 guest trap，不抬成 `result`。规范 `result`/`option` 走 S 系列（[`../scheme/guest-shape.md`](../scheme/guest-shape.md)）。GPU 后端见 [`../blocked-gpu-host.md`](../blocked-gpu-host.md)。
