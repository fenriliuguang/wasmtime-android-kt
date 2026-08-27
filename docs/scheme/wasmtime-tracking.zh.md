# 官方 Wasmtime 依赖追踪

[English](wasmtime-tracking.md) | **中文**

只依赖官方 `wasmtime`（及明确选用的官方附属 crate）。**不**依赖 wasmtime4j。

当前钉死：**47.0.4**（`native/Cargo.toml`）。KPI 是可知、可升级、可回滚，不是追最新 major。

P2 手册（点名）：[`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md)。现行自动序：`python3 ./scripts/product-010-remaining.py`。2026-08-26 检查：上游 latest **48.0.1**；本代 latest **47.0.4**（含 WASIp3 stream / FS GHSA）。**不**升 major。`0.1.0` 队列见 [`product-010.zh.md`](product-010.zh.md)。

major 升级须独立 RFC。细节以英文正文为准。
