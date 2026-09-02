# Gap: WASI 0.3.0 official WIT vs this repo

**English** | [中文](gap-wasi-p3-wit.zh.md)

Named leftovers only. Do **not** auto-cut these. Product subset: [`../scheme/claim-010.md`](../scheme/claim-010.md).

Pin: [WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0).

| Tag | Meaning |
|-----|---------|
| **Smoke** | Product path already ships this shape |
| **Named** | Official 0.3.0 still has it; this repo will not auto-cut it |
| **Out** | Non-goal (NG-4 testsuite, `wasmtime-wasi` crate, 0.2 pollable) |

## Named leftovers

| ID | Official 0.3.0 | Why named-only |
|----|----------------|----------------|
| **G-err** | Full `error-code` enums + err paths | Product cli already has `illegal-byte-sequence` / `io` / `pipe` |
| **G-cmd** | `wasi:cli/command` environment / exit / terminal-* | Product is run + stdio |
| **G-fs-full** | `stat`, directory stream, append, sync, dates | Beyond open + r/w |
| **G-sock-rest** | `listen`, UDP, DNS, sockets `types` merge | Outbound TCP landed |
| **G-http** | Full `service` world, trailers, TLS / https | Body stream + outbound GET landed |

## Coverage (now)

| Package | Degree |
|---------|--------|
| CM stream/future, `wasi:random`, `wasi:clocks` instant | **Smoke** |
| `wasi:cli` stdout/stderr/stdin/run | **Smoke** + one guest-visible err |
| `wasi:filesystem` | **Smoke** directory `open-at` + r/w + `..` → `access` |
| `wasi:sockets` | **Smoke** create + connect + outbound non-loopback IPv4 |
| `wasi:http` | **Smoke** body `stream<u8>` + outbound GET; product linker omits request/response constructors |
