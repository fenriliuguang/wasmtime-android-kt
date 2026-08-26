# P1 WASI 0.3 — 已关闭 2026-08-26

**已关闭。** P1 是 **WASI 0.3 官方形状** 程序（W1–W8 + FS/SK/HT 点名刀 + G-dev）。**不是**完整 WASI 0.3、不是 `wasi-testsuite`、不是 `wasmtime-wasi` crate。

**现行工作：** P2 Wasmtime 钉 — [`docs/agent/wasmtime-p2.md`](../agent/wasmtime-p2.md)。下一刀：`python3 ./scripts/wasmtime-p2-remaining.py`。

**不要重切** W1–W8、P1-FS1–FS4、P1-SK1–SK2、P1-HT1 或 G-dev。点名遗留（G-err、G-cmd、G-fs-full、G-sock-rest、G-http-body、G-http-ctor、G-cli-error）在 [`docs/mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md)，作为**点名才做的未来优化**，不进 `remaining.py` 的 `Next:`。

手册快照：[`p1-wasi-p3-playbook.zh.md`](p1-wasi-p3-playbook.zh.md)。表面快照：[`p1-wasi-p3-surface.zh.md`](p1-wasi-p3-surface.zh.md)。Skill 快照：[`skills/wasi-p3.md`](skills/wasi-p3.md)。

英文：[`p1-wasi-p3.md`](p1-wasi-p3.md)。P0 收口：[`p0-wasi-webgpu.zh.md`](p0-wasi-webgpu.zh.md)。

## 已合入

| 车道 | PR | 声称 |
| --- | --- | --- |
| W1 stream multi-chunk | [#258](https://github.com/wasm3-android/wasm3-android/pull/258) | `wasi:io/streams@0.3.0` `read-stream` + `[stream]` |
| W2 clocks official instant | [#259](https://github.com/wasm3-android/wasm3-android/pull/259) | `wasi:clocks/monotonic-clock@0.3.0` `now: func() -> instant` |
| W3 cli stdout/stderr result | [#260](https://github.com/wasm3-android/wasm3-android/pull/260) | `get-stdout` / `get-stderr` `-> result<…, error>` |
| W4 cli stdin tuple | [#261](https://github.com/wasm3-android/wasm3-android/pull/261) | `get-stdin` `-> tuple<…>` |
| W5 cli command result | [#262](https://github.com/wasm3-android/wasm3-android/pull/262) | `run: func() -> result` |
| W6 filesystem preopen | [#263](https://github.com/wasm3-android/wasm3-android/pull/263) | `wasi:filesystem/preopens@0.3.0` |
| W7 sockets tcp | [#264](https://github.com/wasm3-android/wasm3-android/pull/264) | `wasi:sockets/tcp@0.3.0` |
| W8 http handler | [#265](https://github.com/wasm3-android/wasm3-android/pull/265) | `wasi:http/handler@0.3.0` |
| Gap 表 | [#266](https://github.com/wasm3-android/wasm3-android/pull/266) | WIT vs 宿主 vs smoke 活表 |
| P1-FS1 list tuple | [#267](https://github.com/wasm3-android/wasm3-android/pull/267) | `get-directories` `-> tuple<list<tuple<…>>>` |
| P1-FS2 rw offset | [#268](https://github.com/wasm3-android/wasm3-android/pull/268) | `read`/`write` `offset: filesize` |
| P1-FS3 open-at | [#269](https://github.com/wasm3-android/wasm3-android/pull/269) | 目录 `open-at` |
| P1-FS4 open-at access | [#270](https://github.com/wasm3-android/wasm3-android/pull/270) | `..` → `error-code.access` |
| P1-SK1 create-family | [#271](https://github.com/wasm3-android/wasm3-android/pull/271) | `create-tcp-socket(family) -> result` |
| P1-SK2 connect-addr | [#272](https://github.com/wasm3-android/wasm3-android/pull/272) | `connect(ip-socket-address) -> result` |
| P1-HT1 handle-result | [#273](https://github.com/wasm3-android/wasm3-android/pull/273) | `handle -> result<response, error-code>` |
| G-dev | [#274](https://github.com/wasm3-android/wasm3-android/pull/274) | 十个 WIT 0.3.0 仪器在 **V2458A arm64 Android 16** |

WIT 钉：[`wasi-clocks@0.3.0`](https://github.com/WebAssembly/wasi-clocks/releases/tag/v0.3.0) 至 [`wasi-http@0.3.0`](https://github.com/WebAssembly/wasi-http/releases/tag/v0.3.0)。宿主：`native/src/host/wasi/`。客体：`native/tests/wasi_*_guest.rs`。

## 锁定目标（Smoke）

P1 只锁定 **Smoke**：G-fs-shape、G-fs-open、G-sock-shape、G-http-shape。G-dev 是 **Smoke**。不要声称 **Pass** / 完整 WASI 0.3 / testsuite / Maven。

## 本阶段未关闭

见 [`docs/mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md) §3：

- G-err / G-cmd / G-fs-full / G-sock-rest / G-http-body / G-http-ctor / G-cli-error
- `wasi-testsuite`、`wasmtime-wasi` crate、WASI 0.2 pollable、P0 `wasi:webgpu` 遗留描述符

这些是**点名优化**。**不是** P2 自动队列。

## 仍成立的事实

- `MAX_FLAT_RESULTS = 1`；`result` 判别值 **1 字节**（`i32.load8_u`）；payload align 4。
- 已是 WIT `async func` 不要再加 `canon lower … async`。
- 导入 instance 的 variant case 必须引用已导出的 record（P1-SK2）。
- 导出 `result<own, error>`：instance 必须 **export error-code**；lift 核心返回**对齐的结果指针**（P1-HT1）。
- `rustc` **1.97.1**。禁止 crate-wide `cargo fmt` `cm.rs`。
- 永不向上游 `gh issue create`。

## 下一阶段

**P2** — Wasmtime 钉：可知、可升级、可回滚。手册：[`docs/agent/wasmtime-p2.md`](../agent/wasmtime-p2.md)。
