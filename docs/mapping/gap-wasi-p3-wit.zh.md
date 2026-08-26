# 差距：WASI 0.3.0 官方 WIT ↔ 本仓（P1）

[English](gap-wasi-p3-wit.md) | **中文**

W1–W8 已合入 **包名 smoke**，不是完整 0.3.0 guest 链接。完成目标（已锁定）：**G-fs-shape**、**G-fs-open**、**G-sock-shape**、**G-http-shape**。短刀序与延期项以英文正文为准。

选刀：`.\scripts\wasi-p3-remaining.ps1` / `python3 ./scripts/wasi-p3-remaining.py`。钉 [WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0)。不要重切 W1–W8。

## 完成目标 → 短刀

| 目标 | 短刀（自动序） |
|------|----------------|
| G-fs-shape | **Smoke**（P1-FS1 list + P1-FS2 offset） |
| G-fs-open | P1-FS3 目录 preopen + `open-at` 成功路径 → P1-FS4 guest `..` → `access` |
| G-sock-shape | P1-SK1 `create-tcp-socket(family) -> result` → P1-SK2 `connect(addr) -> result` |
| G-http-shape | P1-HT1 `handle -> result<response, error-code>` |

## 延期（官方 0.3.0，不进 Next）

G-err 全量 error-code；G-cmd command world；G-fs-full stat/目录流；G-sock-rest listen/UDP/DNS；G-http-body / service / 出站；G-http-ctor 去掉测试 constructor；G-dev 真机；G-cli-error。点名：wasi-testsuite、`wasmtime-wasi` crate。
