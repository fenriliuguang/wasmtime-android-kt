# Gap: WASI 0.3.0 official WIT vs this repo (P1)

**English** | [中文](gap-wasi-p3-wit.zh.md)

Living map for **P1 after W1–W8 smokes**. Pin: [WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0) ([overview](https://wasi.dev/releases/wasi-p3)). Playbook: [`../agent/wasi-p3.md`](../agent/wasi-p3.md). Surface: [`../scheme/wasi-p3-surface.md`](../scheme/wasi-p3-surface.md). Next cut: `.\scripts\wasi-p3-remaining.ps1` (or `python3 ./scripts/wasi-p3-remaining.py`).

W1–W8 landed **package-name smokes** (and official **cli/clocks** shapes). They are **not** a full 0.3.0 guest link. This page is the cut queue for the shapes still missing. Do **not** re-cut W1–W8 sentinels.

**Degree**

| Tag | Meaning |
|-----|---------|
| **Goal** | In scope: finish so a 0.3.0-shaped guest can **instantiate** that import/export |
| **Smoke** | W1–W8 already shipped; keep the instrument, promote the WIT |
| **Defer** | Official 0.3.0 still has it; this repo will not auto-cut it |
| **Out** | Non-goal (NG-4 testsuite, `wasmtime-wasi` crate, 0.2 pollable) |

## 1. Completion goals (user-locked)

| Goal | Official 0.3.0 target (this repo’s pin) | W1–W8 now | Short knives (auto order) |
|------|------------------------------------------|-----------|---------------------------|
| **G-fs-shape** | `get-directories` → `list<tuple<descriptor, string>>`; `write-via-stream(data: stream<u8>, offset: filesize) -> future<result<_, error-code>>`; `read-via-stream(offset: filesize) -> tuple<stream<u8>, future<result<_, error-code>>>` | **Smoke** list length 1 + r/w `offset` (smoke uses `0`) | — |
| **G-fs-open** | Preopen is a **directory**; `[method]descriptor.open-at` relative path; sandbox `..` / absolute / NUL → `access` | **Smoke** directory preopen + `open-at` + guest `..` → `access` | — |
| **G-sock-shape** | `create-tcp-socket(ip-address-family) -> result<tcp-socket, error-code>`; `connect: async func(ip-socket-address) -> result<_, error-code>` (loopback still OK); data plane stays stream-plus-future | **Smoke** create family/result (`ipv4`); connect takes loopback `ip-socket-address` (host may ignore port) | — |
| **G-http-shape** | Guest export `handle: async func(request) -> result<response, error-code>` | **Smoke** `handle: async func(own<request>) -> result<own<response>, error-code>` (ok path; root `run` still 200) | — |

Polarity note (filesystem / TCP write): WASI 0.3 **takes** `stream<u8>` and returns a completion `future` (same as cli stdout). W6/W7 already use that direction. G-fs-shape only adds **offset** + official **preopen list**. G-sock-shape does **not** re-litigate write/read polarity.

0.3 sockets consolidated `tcp` / `tcp-create-socket` into `types` on wasi.dev. This repo **keeps the W7 instance names** (`wasi:sockets/tcp@0.3.0`, `tcp-create-socket@0.3.0`) until a named crate/`types` cut. G-sock-shape only promotes **function signatures**.

0.3 HTTP world is `wasi:http/service` (not 0.2 `proxy`). G-http-shape keeps the W8 export name `wasi:http/incoming-handler@0.3.0#handle` and only promotes the **result** signature.

## 2. Short knives (one PR each)

Order is **P1-FS1 → P1-FS2 → P1-FS3 → P1-FS4 → P1-SK1 → P1-SK2 → P1-HT1**. One lane, one PR. Extend the existing fixture + native test + device instrument. No second linker stack. No `wasmtime-wasi`.

| PR | Goal | Sentinel (remaining drops when this leaves the fixture) | DoD |
|----|------|-----------------------------------------------------------|-----|
| **P1-FS1** | G-fs-shape | *(landed — fixture no longer has `gap: get-directories not list tuple`)* | `get-directories` → `list<tuple<own<descriptor>, string>>` (length 1). Guest uses index 0. Native + `WasiFilesystemPreopenInstrumentedTest` still 4-byte `P3FS` |
| **P1-FS2** | G-fs-shape | *(landed — fixture no longer has `gap: read/write no filesize offset`)* | `write-via-stream` / `read-via-stream` take `offset: u64`; smoke uses `0` |
| **P1-FS3** | G-fs-open | *(landed — fixture no longer has `gap: no open-at`)* | Preopen is the sandbox **directory**; `[method]descriptor.open-at` happy path `"p3fs.txt"`; smoke writes/reads the **opened** child |
| **P1-FS4** | G-fs-open | *(landed — fixture no longer has `gap: open-at access not guest-visible`)* | Guest `open-at("..")` yields `error-code.access`. Happy-path instrument still 4-byte `P3FS` |
| **P1-SK1** | G-sock-shape | *(landed — fixture no longer has `gap: create-tcp-socket no address-family`)* | `create-tcp-socket: func(ip-address-family) -> result<tcp-socket, error-code>` (ok + `ipv4` in the smoke) |
| **P1-SK2** | G-sock-shape | *(landed — fixture no longer has `gap: connect no ip-socket-address`)* | `connect: async func(ip-socket-address) -> result<_, error-code>`; guest passes loopback (host may ignore port and keep the echo pair). Still INTERNET + helper-thread connect |
| **P1-HT1** | G-http-shape | *(landed — fixture no longer has `gap: handle not result<response>`)* | `handle: async func(own<request>) -> result<own<response>, error-code>` (ok path). Native calls the official export; root `run` still 200 for `callRunConcurrent` |

PR title: `feat(wasi): L2 <package> <family>` (same playbook). Label `enhancement`. Update this page’s **Goal** row to **Smoke** when the goal’s last knife lands.

## 3. Deferred (official 0.3.0, not auto)

Do **not** put these on `Next:`. Record here only.

| ID | Official 0.3.0 | Why deferred |
|----|----------------|--------------|
| **G-err** | Full `error-code` enums + err paths on cli / fs / sockets / http | W3–W8 are ok-path; variants are a second series |
| **G-cmd** | `wasi:cli/command` imports environment / exit / terminal-* and the fs/sockets worlds | W5 is run+stdio only by design |
| **G-fs-full** | `stat`, `read-directory` stream, append, sync, dates, full error-code, `other(option)` | Beyond open+r/w; likely `wasmtime-wasi` sized |
| **G-sock-rest** | `listen` → `stream<tcp-socket>`; UDP; `ip-name-lookup`; non-loopback; sockets `types` merge | Loopback client smoke is enough for G-sock-shape |
| **G-http-body** | request/response method, path, headers, **body `stream<u8>`**, trailers; `outgoing-handler` / `send`; `wasi:http/service` world; wire/loopback server | G-http-shape is instantiate + status 200 only |
| **G-http-ctor** | Drop `[constructor]request` / `[constructor]response` from the **product** types surface (host supplies `request` when calling `handle`) | W8 constructors are **Fixture** (like webgpu `get-device`). Named follow-up, not P1-HT1 |
| **G-cli-error** | cli `error-code` as `io` / `illegal-byte-sequence` / `pipe` (0.3 `wasi:cli/types`) | W3/W4 use `unknown` only |

## 4. Out of scope

| Item | Tag |
|------|-----|
| Full wasi-testsuite P3 | **Out** (NG-4); named-only |
| Enable `wasmtime-wasi` crate | **Out** until a named size + Android thread review |
| WASI 0.2 `wasi:io` pollable as the 0.3 path | **Out** (NG-8) |
| Re-cut W1–W8 smokes / wasi:webgpu G1–G9 | **Out** |

## 5. Coverage (now)

| Package | Degree |
|---------|--------|
| CM stream/future, `wasi:random`, `wasi:clocks` instant | **Smoke** ≈ official for the landed functions |
| `wasi:cli` stdout/stderr/stdin/run | **Smoke** ≈ official signatures; **Defer** G-err / G-cmd |
| `wasi:filesystem` | **Smoke** G-fs-shape + G-fs-open (list, offset, directory `open-at`, `..` → `access`) |
| `wasi:sockets` | **Smoke** loopback echo + create family/result + connect `ip-socket-address` |
| `wasi:http` | **Smoke** in-process 200 + `handle -> result<response, error-code>` |
| Device (G-dev) | **Smoke** — 10 W1–W8 / P1-* instruments on V2458A arm64 Android 16 (2026-08-26). Cloud still has no device |
