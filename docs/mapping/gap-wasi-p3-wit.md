# Gap: WASI 0.3.0 official WIT vs this repo (P1 leftovers)

**English** | [中文](gap-wasi-p3-wit.zh.md)

**P1 is closed** (2026-08-26). Close-out: [`../archive/p1-wasi-p3.md`](../archive/p1-wasi-p3.md). Current work: [`../agent/product-010.md`](../agent/product-010.md). Surface snapshot: [`../archive/p1-wasi-p3-surface.md`](../archive/p1-wasi-p3-surface.md).

This page keeps **P1 leftover official 0.3.0 shapes**. Do **not** re-cut W1–W8, P1-FS1–FS4, P1-SK1–SK2, P1-HT1, or G-dev. They are **not** `wasmtime-p2-remaining.py` `Next:`.

Rows that [`rfc-l5-productization.md`](../scheme/rfc-l5-productization.md) §7 names (outbound TCP, HTTP **body stream** + outbound, drop product G-http-ctor, guest-visible errors on product cli/IO paths) are **auto** on [`../scheme/product-010.md`](../scheme/product-010.md) / `product-010-remaining.py`. G-cmd full world, G-fs-full, listen/UDP stay **named-only** and are **not** `0.1.0` gates.

Pin: [WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0) ([overview](https://wasi.dev/releases/wasi-p3)).

**Degree**

| Tag | Meaning |
|-----|---------|
| **Goal** | In scope *if the user names it*: finish so a 0.3.0-shaped guest can **instantiate** that import/export |
| **Smoke** | P1 already shipped this locked goal; keep the instrument |
| **Named** | Official 0.3.0 still has it; this repo will not auto-cut it |
| **Out** | Non-goal (NG-4 testsuite, `wasmtime-wasi` crate, 0.2 pollable) |

## 1. Completion goals (user-locked, Smoke)

| Goal | Official 0.3.0 target (this repo’s pin) | Status |
|------|------------------------------------------|--------|
| **G-fs-shape** | `get-directories` → `list<tuple<descriptor, string>>`; `write-via-stream(data: stream<u8>, offset: filesize) -> future<result<_, error-code>>`; `read-via-stream(offset: filesize) -> tuple<stream<u8>, future<result<_, error-code>>>` | **Smoke** list length 1 + r/w `offset` (smoke uses `0`) |
| **G-fs-open** | Preopen is a **directory**; `[method]descriptor.open-at` relative path; sandbox `..` / absolute / NUL → `access` | **Smoke** directory preopen + `open-at` + guest `..` → `access` |
| **G-sock-shape** | `create-tcp-socket(ip-address-family) -> result<tcp-socket, error-code>`; `connect: async func(ip-socket-address) -> result<_, error-code>` (loopback still OK); data plane stays stream-plus-future | **Smoke** create family/result (`ipv4`); connect takes loopback `ip-socket-address` (host may ignore port) |
| **G-http-shape** | Guest export `handle: async func(request) -> result<response, error-code>` | **Smoke** `handle: async func(own<request>) -> result<own<response>, error-code>` (ok path; root `run` still 200) |

Polarity note (filesystem / TCP write): WASI 0.3 **takes** `stream<u8>` and returns a completion `future` (same as cli stdout). W6/W7 already use that direction.

0.3 sockets consolidated `tcp` / `tcp-create-socket` into `types` on wasi.dev. This repo **keeps the W7 instance names** (`wasi:sockets/tcp@0.3.0`, `tcp-create-socket@0.3.0`) until a named crate/`types` cut.

0.3 HTTP world is `wasi:http/service` (not 0.2 `proxy`). G-http-shape keeps the W8 export name `wasi:http/incoming-handler@0.3.0#handle`.

## 2. Landed short knives (do not re-cut)

Order was **P1-FS1 → P1-FS2 → P1-FS3 → P1-FS4 → P1-SK1 → P1-SK2 → P1-HT1**. All landed. Extend an existing fixture only if the user **names** a leftover in §3.

| PR | Goal | Status |
|----|------|--------|
| **P1-FS1** | G-fs-shape | landed — `get-directories` → `list<tuple<own<descriptor>, string>>` |
| **P1-FS2** | G-fs-shape | landed — `write-via-stream` / `read-via-stream` take `offset: u64` |
| **P1-FS3** | G-fs-open | landed — directory preopen + `open-at` happy path |
| **P1-FS4** | G-fs-open | landed — guest `open-at("..")` → `error-code.access` |
| **P1-SK1** | G-sock-shape | landed — `create-tcp-socket(family) -> result` |
| **P1-SK2** | G-sock-shape | landed — `connect(ip-socket-address) -> result` |
| **P1-HT1** | G-http-shape | landed — `handle -> result<response, error-code>` |

## 3. Named leftovers

Do **not** put G-cmd / G-fs-full / listen / UDP on `product-010-remaining` `Next:`. L5 §7 rows (outbound TCP, HTTP body/outbound, G-http-ctor, product cli errors) **are** that script’s auto lanes. Cut only the printed **Next:**.

| ID | Official 0.3.0 | Why named-only vs `0.1.0` |
|----|----------------|----------------|
| **G-err** | Full `error-code` enums + err paths on cli / fs / sockets / http | Full dump named. Product-path cli errors: **P010-CLIERR** |
| **G-cmd** | `wasi:cli/command` imports environment / exit / terminal-* and the fs/sockets worlds | W5 is run+stdio only by design |
| **G-fs-full** | `stat`, `read-directory` stream, append, sync, dates, full error-code, `other(option)` | Beyond open+r/w; likely `wasmtime-wasi` sized |
| **G-sock-rest** | `listen` → `stream<tcp-socket>`; UDP; `ip-name-lookup`; non-loopback; sockets `types` merge | listen/UDP/DNS **named-only**. Non-loopback outbound is **P010-TCP** |
| **G-http-body** | request/response method, path, headers, **body `stream<u8>`**, trailers; `outgoing-handler` / `send`; `wasi:http/service` world; wire/loopback server | **P010-HBODY / HOUT** auto. Full `service` world / trailers still named |
| **G-http-ctor** | Drop `[constructor]request` / `[constructor]response` from the **product** types surface (host supplies `request` when calling `handle`) | **P010-HCTOR** auto |
| **G-cli-error** | cli `error-code` as `io` / `illegal-byte-sequence` / `pipe` (0.3 `wasi:cli/types`) | **P010-CLIERR** as needed on product stdio/`run`; full enum dump named |

## 4. Out of scope

| Item | Tag |
|------|-----|
| Full wasi-testsuite P3 | **Out** (NG-4); named-only |
| Enable `wasmtime-wasi` crate | **Out** until a named size + Android thread review |
| WASI 0.2 `wasi:io` pollable as the 0.3 path | **Out** (NG-8) |
| Re-cut W1–W8 smokes / wasi:webgpu G1–G9 / P1 auto knives | **Out** |

## 5. Coverage (now)

| Package | Degree |
|---------|--------|
| CM stream/future, `wasi:random`, `wasi:clocks` instant | **Smoke** ≈ official for the landed functions |
| `wasi:cli` stdout/stderr/stdin/run | **Smoke** ≈ official signatures; **Named** G-err / G-cmd |
| `wasi:filesystem` | **Smoke** G-fs-shape + G-fs-open (list, offset, directory `open-at`, `..` → `access`) |
| `wasi:sockets` | **Smoke** loopback echo + create family/result + connect `ip-socket-address` |
| `wasi:http` | **Smoke** in-process 200 + `handle -> result<response, error-code>` |
| Device (G-dev) | **Smoke** — 10 W1–W8 / P1-* instruments on V2458A arm64 Android 16 (2026-08-26). Cloud still has no device |
