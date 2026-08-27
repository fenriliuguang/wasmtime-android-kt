# `0.1.0` 宣称表（不是 CTS）

[English](claim-010.md) | **中文**

L5 [`rfc-l5-productization.md`](rfc-l5-productization.md) §7–§8 的发布说明形 **产品子集**。与英文冲突时以英文为准。**不是**合规宣称（NG-4 / NG-5）。坐标 **`0.1.0`**。发布 CI：[`.github/workflows/publish.yml`](../../.github/workflows/publish.yml)（secrets / arm64 `.so` 缺失时不要强发）。

## 一句话

第三方可以依赖：钉住的 **`wasi:webgpu@0.3.0-rc.2` 绝大多数 `[method]` 能 instantiate**、webgpu 应用需要的 **WASI 0.3 IO/网络子集**、以及 **完整 `wasi-gfx` 上屏循环**（产品 adapter/device + vsync）。在 **P010-GFXB** / **P010-GFXV** 前，循环只是两帧预缓冲 + fixture `get-device`。在 **P010-DEMO** 前：本表无真机行，入口 README 无仓外 demo 链接（引入链接即视为存在，不把 demo 做进本仓）。androidx 空洞与 named-only WASI 剩项公开列出，不当静默丢字段。

## `wasi:webgpu`

钉 WIT 里 224 个 resource `[method]` 均已在 `native/src/cm.rs` 注册（guest import 可链接）。未接线时 `request-adapter` 为 **`none`**。androidx `1.0.0-alpha05` 仍缺的槽位（shader `compilation-hints`、canvas `color-space` / `tone-mapping`）是 **Record** 不是 Dawn：见 [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md)。Fixture `get-gpu` / `get-device` 不是产品表面。不是 CTS。

## WASI 产品子集 vs 点名

| 包 | `0.1.0` 产品 | 点名（非本门禁） |
|----|--------------|------------------|
| clocks / random / stream | 已落地函数 | — |
| cli | stdio + `run` + NUL → `illegal-byte-sequence` | G-err 全枚举、G-cmd |
| filesystem | 预开目录 + `open-at` + 读写；`..` → `access` | G-fs-full |
| sockets | 出站 TCP 拨非回环 IPv4 | listen / UDP / DNS |
| http | body `stream<u8>` + 线上 GET；产品无 request/response 构造器 | service 世界、TLS |
| gfx | `surface@0.2.0` + 两帧预缓冲 present；**完整循环待 P010-GFXB / P010-GFXV**；最后 **P010-DEMO**（README 仓外 demo + 真机行） | 完整桌面 gfx |

不宣称 `wasmtime-wasi`、不宣称本仓 1.0。发布 workflow 已落地（P010-PUB）；secrets 缺失时不要强发 Central。
