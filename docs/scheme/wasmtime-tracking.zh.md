# 官方 Wasmtime 依赖追踪

[English](wasmtime-tracking.md) | **中文**

只依赖官方 `wasmtime`（及明确选用的官方附属 crate）。**不**依赖 wasmtime4j。

当前钉死：**47.0.2**（`native/Cargo.toml`）。KPI 是可知、可升级、可回滚，不是追最新 major。

major 升级须独立 RFC。细节以英文正文为准。
