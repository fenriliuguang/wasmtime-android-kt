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
| **G-http** | Full `service` world, trailers, TLS / https, remaining handler types | landed (L-HTTP-FIELDS … L-HTTP-SVC); not a listen HTTP server / wasi-testsuite |

## Coverage (now)

| Package | Degree |
|---------|--------|
| CM stream/future, `wasi:random`, `wasi:clocks` instant | **Smoke** |
| `wasi:cli` stdout/stderr/stdin/run | **Smoke** + official `error-code` (`io` / `illegal-byte-sequence` / `pipe`); NUL → `illegal-byte-sequence`; invalid UTF-8 → `io`; `environment` `get-environment` / `get-arguments` (TMPDIR only; arguments empty); `exit` completes `run` with official `result` (does not kill ART); `terminal-*` `get-terminal-*` is `none` (not a fake TTY) |
| `wasi:filesystem` | **Smoke** + official `error-code` variant; `..` → `access`; missing descriptor → `bad-descriptor`; r/w IO → `io` / `is-directory` (no `unknown`); `stat` / `stat-at` on sandbox descriptor; `read-directory` as a CM stream (omit `.` / `..`); `append-via-stream`; `sync` / `sync-data`; `set-times` / `set-times-at` (sandbox files) |
| `wasi:sockets` | **Smoke** + official `error-code` variant; IPv6 create → `not-supported`; failed connect mapped off `unknown`; TCP bind / listen / accept **loopback only** (non-loopback bind → `access-denied`; helper thread, not ART main); UDP create / bind / send / receive **loopback only** (non-loopback bind/send → `access-denied`; helper thread); `ip-name-lookup` `resolve-addresses` on a helper thread (ipv4 list; empty name → `invalid-argument`) |
| `wasi:http` | **Smoke** body `stream<u8>` + outbound GET; official `error-code` variant; empty authority → `HTTP-request-URI-invalid`; https on `client.send` via **rustls** (helper thread, not ART main); `fields` + `request`/`response` `get-headers`; `consume-body` trailers `option` (`none`); incoming-handler types `get-method` / `get-path-with-query` / `get-scheme` / `get-authority` / `set-status-code` (not a listen HTTP server); product linker omits request/response constructors |
