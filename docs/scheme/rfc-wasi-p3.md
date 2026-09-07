# RFC: WASI 0.3 leftover fill vs “completeness”

**Status: Draft** · 2026-09-05 · design discussion only (no implementation on this branch)

English is canonical. Short Chinese: [`rfc-wasi-p3.zh.md`](rfc-wasi-p3.zh.md).

Accepted product policy stays in [`rfc.md`](rfc.md) (product **subset**, not wasi-testsuite). This file is **not** accepted until it is merged into that RFC or withdrawn. Sibling: [`rfc-threads.md`](rfc-threads.md) (Draft). Implementation track for named leftovers: long branch **`cursor/wasi-p3-leftover-b677`**.

## 1. Problem

`0.1.2` ships a **smoke subset** of WASI 0.3 ([`claim-010.md`](claim-010.md)). Official 0.3.0 still has named leftovers ([`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md)): **G-err**, **G-cmd**, **G-fs-full**, **G-sock-rest**, **G-http**.

NG-4 forbids treating “all WASI 0.3 worlds” or “full wasi-testsuite P3” as the **single KPI**. Adding `wasmtime-wasi` still needs a **size + Android thread** note. “Complete WASI 0.3” collides with those rules unless this RFC says what “complete” means here.

Host today is the **in-tree thin host** in `native/src/cm.rs`, not `wasmtime-wasi`.

## 2. What already shipped (do not re-cut)

| Package | Product smoke |
|--------|----------------|
| CM stream/future, `wasi:random`, `wasi:clocks` instant | Landed functions |
| `wasi:cli` | stdin/stdout/stderr + `run`; NUL → `illegal-byte-sequence` |
| `wasi:filesystem` | Directory preopen + `open-at` + r/w; `..` → `access` |
| `wasi:sockets` | Outbound TCP IPv4 (loopback echo + real dial) |
| `wasi:http` | Body `stream<u8>` + outbound GET; product linker omits request/response constructors |

Do **not** re-cut W1–W8 / P1-FS1–FS4 / P1-SK1–SK2 / P1-HT1 as those queues.

## 3. Options

| ID | Proposal | Notes |
|----|----------|-------|
| **A** | Keep leftovers **named-only**; cut only when an out-of-tree example hits a missing import | Matches current gap page. Slowest fill. |
| **B** | Thin-host **living leftover queue** | Fill G-* as one-lane commits on `cursor/wasi-p3-leftover-b677`. Still **not** a testsuite KPI. Still **not** `wasmtime-wasi`. |
| **C** | Switch the WASI surface to **`wasmtime-wasi`** | Needs size + Android thread (GpuThread / 8 MiB pump / hitch). Adjacent to DG-3 (full cloud/CLI distro). |
| **D** | Full **wasi-testsuite** P3 as the DoD | **Reject.** Repeals NG-4. Cloud has no device; Android sandbox is not a Unix preview. |

**Strawman (not accepted as a completeness claim):** **B**. Filling named leftovers is the next implementation track. It does **not** license a “full WASI 0.3” or WASI-1.0 marketing line (NG-5 still holds). Option C is a later RFC with numbers. Option D stays out.

## 4. What B includes / excludes

**Includes (named leftovers → `L-*` lanes):**

- **G-err** — official `error-code` enums + guest-visible err paths (cli / fs / sockets / http)
- **G-cmd** — `environment`, `exit`, `terminal-*` (Android: terminal may be `none`)
- **G-fs-full** — `stat` / directory stream / append / sync / dates (sandbox unchanged)
- **G-sock-rest** — listen / UDP / DNS; listen default remains sandbox-tight
- **G-http** — fields/headers, trailers, TLS/https, remaining `service` / handler shape (still not a listen HTTP server)

**Still Out (unless a later RFC):**

- wasi-testsuite as the merge gate
- `wasmtime-wasi` crate
- WASI 0.2 pollable
- `wasi:clocks` timezone (not in the named leftover table)
- Guest wasm threads ([`rfc-threads.md`](rfc-threads.md))
- Benchmarks (deferred)
- This-repo 1.0.0

## 5. `wasmtime-wasi` bar (C, not leftover)

A leftover lane must **not** add `wasmtime-wasi`. If C is chosen later, the changelog must record:

- arm64 `libwasmtime_android_kt.so` size before / after
- Which thread runs the WASI preview (must not steal GpuThread or block ART main)
- Hitch: leftover IO must not stack on `on-frame` the way `onSubmittedWorkDone` did

Until those numbers exist, leftover stays thin-host.

## 6. Acceptance when this RFC is accepted

- Amend [`rfc.md`](rfc.md) §4 / [`claim-010.md`](claim-010.md) / [`non-goals.md`](non-goals.md): NG-4 **stays** (testsuite is not the KPI). Named leftovers become a living queue, not “never auto-cut.”
- Point remaining at `python3 ./scripts/wasi-p3-leftover-remaining.py`.
- Do not claim CTS / complete WASI 0.3 / WASI 1.0 distro.
- Still never file upstream GitHub issues.
