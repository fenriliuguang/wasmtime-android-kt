# P1 WASI 0.3 — closed 2026-08-26

**Closed.** P1 is the **WASI 0.3 official-shape** program (W1–W8 + FS/SK/HT named cuts + G-dev). It is **not** full WASI 0.3, not `wasi-testsuite`, not a `wasmtime-wasi` crate.

**Current work:** P2 Wasmtime pin — [`docs/agent/wasmtime-p2.md`](../agent/wasmtime-p2.md). Next PR: `python3 ./scripts/wasmtime-p2-remaining.py`.

**Do not re-cut** W1–W8, P1-FS1–FS4, P1-SK1–SK2, P1-HT1, or G-dev. Named leftovers (G-err, G-cmd, G-fs-full, G-sock-rest, G-http-body, G-http-ctor, G-cli-error) live in [`docs/mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md) as **point-named future optimizations**, not as `remaining.py` `Next:`.

Playbook snapshot: [`p1-wasi-p3-playbook.md`](p1-wasi-p3-playbook.md). Surface snapshot: [`p1-wasi-p3-surface.md`](p1-wasi-p3-surface.md). Skill snapshot: [`skills/wasi-p3.md`](skills/wasi-p3.md).

Chinese: [`p1-wasi-p3.zh.md`](p1-wasi-p3.zh.md). P0 close-out: [`p0-wasi-webgpu.md`](p0-wasi-webgpu.md).

## What shipped

| Lane | PR | Claim |
| --- | --- | --- |
| W1 stream multi-chunk | [#258](https://github.com/wasm3-android/wasm3-android/pull/258) | `wasi:io/streams@0.3.0` `read-stream` + `[stream]` |
| W2 clocks official instant | [#259](https://github.com/wasm3-android/wasm3-android/pull/259) | `wasi:clocks/monotonic-clock@0.3.0` `now: func() -> instant` |
| W3 cli stdout/stderr result | [#260](https://github.com/wasm3-android/wasm3-android/pull/260) | `get-stdout` / `get-stderr` `-> result<…, error>` |
| W4 cli stdin tuple | [#261](https://github.com/wasm3-android/wasm3-android/pull/261) | `get-stdin` `-> tuple<…>` |
| W5 cli command result | [#262](https://github.com/wasm3-android/wasm3-android/pull/262) | `run: func() -> result` |
| W6 filesystem preopen | [#263](https://github.com/wasm3-android/wasm3-android/pull/263) | `wasi:filesystem/preopens@0.3.0` |
| W7 sockets tcp | [#264](https://github.com/wasm3-android/wasm3-android/pull/264) | `wasi:sockets/tcp@0.3.0` |
| W8 http handler | [#265](https://github.com/wasm3-android/wasm3-android/pull/265) | `wasi:http/handler@0.3.0` |
| Gap table | [#266](https://github.com/wasm3-android/wasm3-android/pull/266) | Living WIT vs host vs smoke map |
| P1-FS1 list tuple | [#267](https://github.com/wasm3-android/wasm3-android/pull/267) | `get-directories` `-> tuple<list<tuple<…>>>` |
| P1-FS2 rw offset | [#268](https://github.com/wasm3-android/wasm3-android/pull/268) | `read`/`write` `offset: filesize` |
| P1-FS3 open-at | [#269](https://github.com/wasm3-android/wasm3-android/pull/269) | Directory `open-at` |
| P1-FS4 open-at access | [#270](https://github.com/wasm3-android/wasm3-android/pull/270) | `..` → `error-code.access` |
| P1-SK1 create-family | [#271](https://github.com/wasm3-android/wasm3-android/pull/271) | `create-tcp-socket(family) -> result` |
| P1-SK2 connect-addr | [#272](https://github.com/wasm3-android/wasm3-android/pull/272) | `connect(ip-socket-address) -> result` |
| P1-HT1 handle-result | [#273](https://github.com/wasm3-android/wasm3-android/pull/273) | `handle -> result<response, error-code>` |
| G-dev | [#274](https://github.com/wasm3-android/wasm3-android/pull/274) | Ten WIT 0.3.0 instruments on **V2458A arm64 Android 16** |

WIT pin: [`wasi-clocks@0.3.0`](https://github.com/WebAssembly/wasi-clocks/releases/tag/v0.3.0) through [`wasi-http@0.3.0`](https://github.com/WebAssembly/wasi-http/releases/tag/v0.3.0). Host: `native/src/host/wasi/`. Guest: `native/tests/wasi_*_guest.rs`.

## Locked targets (Smoke)

P1 locked **Smoke** only: G-fs-shape, G-fs-open, G-sock-shape, G-http-shape. G-dev is **Smoke**. Do not claim **Pass** / full WASI 0.3 / testsuite / Maven.

## What this phase did **not** close

See [`docs/mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md) §3:

- G-err / G-cmd / G-fs-full / G-sock-rest / G-http-body / G-http-ctor / G-cli-error
- `wasi-testsuite`, `wasmtime-wasi` crate, WASI 0.2 pollable, P0 `wasi:webgpu` leftover descriptors

Those are **named optimizations**. They are **not** the P2 automatic queue.

## Facts that remain true

- `MAX_FLAT_RESULTS = 1`; `result` discriminant is **1 byte** (`i32.load8_u`); payload align 4.
- Do not add `canon lower … async` on WIT that is already `async func`.
- Import-instance variant cases must name already-exported records (P1-SK2).
- Export `result<own, error>`: instance must **export the error-code type**; lift core returns an **aligned result pointer** (P1-HT1).
- `rustc` **1.97.1**. No crate-wide `cargo fmt` of `cm.rs`.
- Never file upstream GitHub issues (`gh issue create`).

## Next phase

**P2** — Wasmtime pin: known, upgradeable, rollbackable. Playbook: [`docs/agent/wasmtime-p2.md`](../agent/wasmtime-p2.md).
