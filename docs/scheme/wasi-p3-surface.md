# WASI 0.3 surface (P1)

**English** | [中文](wasi-p3-surface.zh.md)

Companion to [`long-term-plan.md`](long-term-plan.md) **P1**.  
Spec baseline: [WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0) (approved 2026-06-11) · [wasi.dev/releases/wasi-p3](https://wasi.dev/releases/wasi-p3) · [BA: WASI 0.3 Launched](https://bytecodealliance.org/articles/WASI-0.3).

**This page is a scheduling surface, not a full-implementation promise.**

## 1. Stance

WASI 0.3 moved async from `wasi:io` **down** into the Component Model:

| WASI 0.2 | WASI 0.3 |
|----------|----------|
| `resource pollable` | `future<T>` |
| `input-stream` / `output-stream` | `stream<T>` (write direction flipped) |
| `poll` / `subscribe` | runtime `await` |
| `start-foo` / `finish-foo` | `async func` |

This repo already verified **future + async host import** (M2). Long-term: land the ratified primitives and **on-demand** package subsets on the Android JNI/Kotlin thin L1, serving P0 (`wasi:webgpu`) and ordinary guests.

**Do not** treat “pass all wasi-testsuite P3” or “implement every world” as the only success metric.

## 2. Layers

```text
Primitives (this repo’s L1)     async func · future · stream · scheduler pump
Package (host glue)             clocks / random / cli / filesystem / sockets / http …
World (composition)             wasi:cli/command · wasi:http/service · …
```

A primitive gap blocks every 0.3 guest. Packages / worlds open **when a guest or webgpu needs them**.

## 3. Primitive priority

| ID | Capability | Vs current tree | Priority | Notes |
|----|------------|-----------------|----------|-------|
| P3-PRIM-1 | `async func` host/guest | M2 concurrent register + `run_concurrent` | **Keep / productize** | Document thread pump; concurrent callbacks |
| P3-PRIM-2 | `future<T>` create/complete/reject | M2 oneshot path works | **Keep / extend** | Many futures, error complete, lifetime |
| P3-PRIM-3 | `stream<T>` read/write | **Read+write smoke** (`fixtures/p3`) | **Keep / extend** | Multi-chunk, backpressure, errors still open |
| P3-PRIM-4 | stream+future completion | **Minimal** (`take` → `future<u32>` bytes) | With P3-PRIM-3 | Full WASI error surface is a later slice |
| P3-PRIM-5 | Write-direction flip (host consumes guest `stream`) | **Smoke** (`fixtures/p3/stream_write`) | **Keep** | stdout / network send can hang off this |
| P3-PRIM-6 | 0.2 polyfill (optional) | Not done | Low | Upstream/runtime may cover; does not block P0 |

**Admission:** every primitive slice needs a reproducible test (prefer Android instruments; desktop JVM may assist) and an update to [`../mapping/threading-android.md`](../mapping/threading-android.md) when the pump changes.

## 4. Package / world priority

Official 0.3 core (from wasi.dev):

| Package / world | Priority here | Open when (all must hold) |
|-----------------|---------------|---------------------------|
| **CM primitives** (table above) | **Required base** | L1 stack |
| `wasi:clocks` | **High** | Smokes already land `monotonic-clock.now` / `wait-for` / `wait-until` / `system-clock.now` / resolutions (`fixtures/wasi/…`, pin `@0.3.0`). `system-clock.now` is transitional `u64` unix seconds (not official `instant` record); timezone is a later slice |
| `wasi:random` | **High** | `get-random-u64` + `get-random-bytes` smokes (`list<u8>`, host cap 4096) |
| `wasi:cli` stdio (stream+future) | **Medium-high** | stdout/stderr `write-via-stream` + stdin `read-via-stream` smokes; transitional `future<u32>` byte counts; stdin transitional `func() -> stream<u8>` (not official tuple+`result`) |
| `wasi:cli/command` (`async run`) | **Medium** | `async run` smoke; transitional root `run: async func() -> u32` (0=ok). Not a full command world |
| `wasi:filesystem` | **Medium** | Guest is blocked; Android sandbox path policy first |
| `wasi:sockets` | **Medium-low** | Network permission + threading RFC |
| `wasi:http` | **Medium-low** | Not the Android first-ship story; desktop may lead |
| `wasi:io@0.2` | **Do not** as 0.3 | Package removed; do not re-implement pollable as the main path |
| Other unlisted proposal packages | **Default no** | Except [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) |

`wasi:webgpu` is **not** a ratified WASI 0.3 core package — it is a **proposal**. Product narrative may rank it above most ratified packages; **engineering still depends on primitives** (especially async / later stream).

## 5. Ownership

| Work | Lands in |
|------|----------|
| Engine config, linker registration, future/stream JNI, scheduler pump | **This repo** |
| Device (GPU / Surface) | This repo’s pluggable host ([`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md)); default Dawn in `:host-dawn`. Today still unpublished — [`../blocked-gpu-host.md`](../blocked-gpu-host.md) |
| Pure-logic WASI (clocks/random/cli subset) | Prefer Kotlin host stubs here; crate split later if needed |
| Full HTTP / sockets | Separate RFC; evaluate exposing `wasmtime-wasi` via JNI only after Android thread + size review |

Forbidden: pile business policy in Rust JNI; restore Latch / sync-compat fake async to “make progress”.

## 6. Tests and claims

| Layer | Use |
|-------|-----|
| In-tree fixtures | Close primitives and tiny packages |
| [wasi-testsuite](https://github.com/WebAssembly/wasi-testsuite) P3 subset | **Optional** regression; pick cases that intersect the implemented surface |
| Full suite / certification | **Not** a near-term gate |

Say: “this repo supports WASI 0.3 primitive X / package Y subset (WIT 0.3.0 pin).”  
Do not say: “full WASI 0.3 compatible runtime” without a compliance RFC.

## 7. Revisions

Changing one package’s priority by one notch: update **that row** + a `changelog/unreleased/` fragment. Do not rewrite other rows. Live “next cut” lives on the [Project](https://github.com/users/fenriliuguang/projects/1), not as a running list here.

Promoting a proposal to P0 alongside webgpu requires a long-term-plan RFC.
