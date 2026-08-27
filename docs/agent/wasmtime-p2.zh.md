# Agent 手册：Wasmtime 钉（P2）

[English](wasmtime-p2.md) | **中文**

P0 `wasi:webgpu` **已关闭**（[`../archive/p0-wasi-webgpu.zh.md`](../archive/p0-wasi-webgpu.zh.md)）。P1 WASI 0.3 官方形状 **已关闭**（[`../archive/p1-wasi-p3.zh.md`](../archive/p1-wasi-p3.zh.md)）。不要重切 W1–W8、P1-FS1–FS4、P1-SK1–SK2、P1-HT1、G-dev，或 wasi:webgpu G1–G9 / F1–F9 / guest-pipeline / WG-6。

本队列保持 **官方 `wasmtime` 钉可知、可升级、可回滚**。追踪表：[`../scheme/wasmtime-tracking.zh.md`](../scheme/wasmtime-tracking.zh.md)。KPI **不是**“永远追最新 major”。一车道一 PR。

现行自动序是 **`0.1.0`**（[`product-010.md`](product-010.md)）。本手册仅在用户点名 P2 / Wasmtime 钉时使用。L5 §7 行走 `product-010-remaining`。

## 目标

第三人能读追踪表 §2 / §3，知道：钉、上次检查日期、检查时上游 latest、下一步允许的升级路径（patch vs RFC）。不是 `wasmtime-wasi` crate。不是再切 WASI 0.3。不是 Maven Central（Central 等到 **P010-PUB**）。

当前钉：**47.0.4**（`native/Cargo.toml`）。Dependabot **忽略 major**。

## 选刀

用户点名则只做那一刀。否则：

```powershell
.\scripts\wasmtime-p2-remaining.ps1
```

无 `pwsh`：`python3 ./scripts/wasmtime-p2-remaining.py`（同样 `--all`）。

只做打印的 **Next:** 行。

## 硬禁

- 不要重切 P0 / P1 自动刀。GPU 与 WASI 遗留页是文档 / 点名。
- 不加 `wasmtime-wasi`，除非该 PR changelog 写了 size + Android 线程审查。
- 不引入 wasmtime4j。
- **major**（47 → 48+）必须先写短升级 RFC（追踪表 §4.1），不能当普通 Dependabot / remaining 刀合入。
- 纯文档 eval **不要**改 `native/Cargo.toml`，除非本 PR **就是** patch 升级。
- 钉/升级 PR **不要**改枢纽：根 `README.md` / `README.zh.md`、`CHANGELOG.md`、`.github/workflows/ci.yml`、`CONTRIBUTING.md`。（改现行队列的政策 PR 可以改 README 计划表。）
- 禁止 crate-wide `cargo fmt`。`rustc` **1.97.1**。
- 永不向上游 `gh issue create`。

复制源、白名单、窄测以[英文正文](wasmtime-p2.md)为准。
