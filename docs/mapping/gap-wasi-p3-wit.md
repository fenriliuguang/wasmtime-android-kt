# Gap: WASI 0.3.0 official WIT vs this repo

**English** | [中文](gap-wasi-p3-wit.zh.md)

Product subset: [`../scheme/claim-010.md`](../scheme/claim-010.md). After `0.1.2`, named leftovers are a **living leftover queue** on **`cursor/wasi-p3-leftover-b677`** ([`../scheme/wasi-p3-leftover.md`](../scheme/wasi-p3-leftover.md); `python3 ./scripts/wasi-p3-leftover-remaining.py`). Draft policy: [`../scheme/rfc-wasi-p3.md`](../scheme/rfc-wasi-p3.md). This is **not** a wasi-testsuite KPI (NG-4) and **not** `wasmtime-wasi`.

Pin: [WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0).

| Tag | Meaning |
|-----|---------|
| **Smoke** | Product path already ships this shape |
| **Leftover** | Official 0.3.0 still has it; living `L-*` queue (not wasi-testsuite) |
| **Out** | Non-goal (NG-4 testsuite, `wasmtime-wasi` crate, 0.2 pollable) |

## Named leftovers → `L-*` lanes

| ID | Official 0.3.0 | First `L-*` |
|----|----------------|-------------|
| **G-err** | Full `error-code` enums + err paths | **L-ERR-CLI** (then FS / SOCK / HTTP) |
| **G-cmd** | `wasi:cli/command` environment / exit / terminal-* | L-CMD-ENV |
| **G-fs-full** | `stat`, directory stream, append, sync, dates | L-FS-STAT |
| **G-sock-rest** | `listen`, UDP, DNS, sockets `types` merge | L-SOCK-LISTEN |
| **G-http** | Full `service` world, trailers, TLS / https | L-HTTP-FIELDS |

## Coverage (now)

| Package | Degree |
|---------|--------|
| CM stream/future, `wasi:random`, `wasi:clocks` instant | **Smoke** |
| `wasi:cli` stdout/stderr/stdin/run | **Smoke** + official `error-code` (`io` / `illegal-byte-sequence` / `pipe`); NUL → `illegal-byte-sequence`; invalid UTF-8 → `io` |
| `wasi:filesystem` | **Smoke** + official `error-code` variant; `..` → `access`; missing descriptor → `bad-descriptor`; r/w IO → `io` / `is-directory` (no `unknown`) |
| `wasi:sockets` | **Smoke** create + connect + outbound non-loopback IPv4 |
| `wasi:http` | **Smoke** body `stream<u8>` + outbound GET; product linker omits request/response constructors |
