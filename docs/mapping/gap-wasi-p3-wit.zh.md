# 差距：WASI 0.3.0 官方 WIT ↔ 本仓（P1 遗留）

[English](gap-wasi-p3-wit.md) | **中文**

**P1 已关闭**（2026-08-26）。收口：[`../archive/p1-wasi-p3.zh.md`](../archive/p1-wasi-p3.zh.md)。现行：[`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md)。表面快照：[`../archive/p1-wasi-p3-surface.zh.md`](../archive/p1-wasi-p3-surface.zh.md)。

本页保留 **P1 遗留的官方 0.3.0 形状**，作为**点名才做的未来优化**。不进 `wasmtime-p2-remaining.py` 的 `Next:`。不要自动切。不要重切 W1–W8、P1-FS1–FS4、P1-SK1–SK2、P1-HT1 或 G-dev。

钉 [WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0)。完成目标（已锁定 **Smoke**）：**G-fs-shape**、**G-fs-open**、**G-sock-shape**、**G-http-shape**。

## 完成目标

| 目标 | 状态 |
|------|------|
| G-fs-shape | **Smoke**（P1-FS1 list + P1-FS2 offset） |
| G-fs-open | **Smoke**（P1-FS3 `open-at` + P1-FS4 `..` → `access`） |
| G-sock-shape | **Smoke**（P1-SK1 family+result + P1-SK2 `connect(addr) -> result`） |
| G-http-shape | **Smoke**（P1-HT1 `handle -> result<response, error-code>`） |

## 点名优化（不进 Next）

G-err 全量 error-code；G-cmd command world；G-fs-full stat/目录流；G-sock-rest listen/UDP/DNS；G-http-body / service / 出站；G-http-ctor 去掉测试 constructor；G-cli-error。G-dev 已录（V2458A arm64，2026-08-26）。点名：wasi-testsuite、`wasmtime-wasi` crate。
