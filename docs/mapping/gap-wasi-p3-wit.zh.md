# 差距：WASI 0.3.0 官方 WIT ↔ 本仓（P1 遗留）

[English](gap-wasi-p3-wit.md) | **中文**

**P1 已关闭**（2026-08-26）。收口：[`../archive/p1-wasi-p3.zh.md`](../archive/p1-wasi-p3.zh.md)。现行：[`../agent/product-010.md`](../agent/product-010.md)。表面快照：[`../archive/p1-wasi-p3-surface.zh.md`](../archive/p1-wasi-p3-surface.zh.md)。

本页保留 **P1 遗留的官方 0.3.0 形状**。不要重切 W1–W8、P1-FS1–FS4、P1-SK1–SK2、P1-HT1 或 G-dev。不进 `wasmtime-p2-remaining.py` 的 `Next:`。

L5 §7 点名的行（出站 TCP、HTTP body stream + 出站、产品面拿掉 G-http-ctor、产品路径上的 guest 可见错误）走 **`product-010-remaining.py` 自动序**。G-cmd 全 world、G-fs-full、listen/UDP **不是** 0.1.0 门禁。

钉 [WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0)。完成目标（已锁定 **Smoke**）：**G-fs-shape**、**G-fs-open**、**G-sock-shape**、**G-http-shape**。

## 完成目标

| 目标 | 状态 |
|------|------|
| G-fs-shape | **Smoke**（P1-FS1 list + P1-FS2 offset） |
| G-fs-open | **Smoke**（P1-FS3 `open-at` + P1-FS4 `..` → `access`） |
| G-sock-shape | **Smoke**（P1-SK1 family+result + P1-SK2 `connect(addr) -> result`） |
| G-http-shape | **Smoke**（P1-HT1 `handle -> result<response, error-code>`） |

## 点名优化（不进 P2 Next）

G-cmd command world；G-fs-full stat/目录流；G-sock-rest 的 listen/UDP/DNS（非回环出站已 P010-TCP）。G-dev 已录。点名：wasi-testsuite、`wasmtime-wasi` crate。

L5 §7 行（出站 TCP、HTTP body/出站、G-http-ctor、产品路径 cli 错误）走 **`product-010-remaining.py`**，不是本表的「永不自动」。HTTP body `stream<u8>` 已 P010-HBODY；出站 `client.send` 已 P010-HOUT。
