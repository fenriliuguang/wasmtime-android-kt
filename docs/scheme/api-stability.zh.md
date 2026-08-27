# API 稳定性（experimental）

[English](api-stability.md) | **中文**

上游 1.0 门禁前长期 **`0.x`**（破坏升 MINOR，学 three.js）。本仓 **1.0.0** 见 L5 §6，不是日历目标。本地坐标 `0.1.0-experimental`；**`0.1.0` 门禁前不发 Central**。`ExperimentalHostCallbacks` 已移出 `runtime-api` 公共 SPI（P010-SPI）。双轨接线：`setWebGpuBackend` 为稳定合同；`Store.createWithDiscoveredBackend` 为默认 bundle 便利（P010-DISC）。产品 `Linker.create` 不含 fixture 构造器 `get-gpu` / `get-device` / `get-gpu-error` / `get-device-lost-info`（P010-FIX）。产品 cli `error-code` 含 `io` / `illegal-byte-sequence` / `pipe`；stdout/stderr 写 NUL 为 guest 可见 `illegal-byte-sequence`（P010-CLIERR）。产品 `tcp-socket.connect` 拨非回环 IPv4；默认不 listen / UDP（P010-TCP）。产品 http types 有 body `stream<u8>`（`consume-body` / `response.new`；P010-HBODY）。产品 `wasi:http/client#send` 为线上 HTTP/1.1 GET（P010-HOUT；本刀无 TLS crate）。产品 `Linker.create` 不含 HTTP `[constructor]request` / `[constructor]response`；host 调 `handle` 时提供 request（P010-HCTOR）。不宣称 CTS。Guest 钉 `wasi:webgpu@0.3.0-rc.2`。
