# WASI 0.3 surface (P1)

**English** | [中文](wasi-p3-surface.zh.md)

Companion to [`long-term-plan.md`](long-term-plan.md) **P1**. Playbook: [`../agent/wasi-p3.md`](../agent/wasi-p3.md). Next cut: `.\scripts\wasi-p3-remaining.ps1`.

Spec: [WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0). **This page is the scheduling surface, not a wasi-testsuite promise** (NG-4).

## Stance

WASI 0.3 moved async into the Component Model (`async func`, `future<T>`, `stream<T>`). This repo hosts those with true CM async on Android JNI/Kotlin. Do not restore Latch fake-async. Do not re-implement `wasi:io@0.2` pollable as the 0.3 path.

`wasi:webgpu` is a **proposal** (P0, closed). It is not a ratified 0.3 core package.

## Now vs P1 lanes

| Area | Landed | P1 remaining (W1–W8) |
|------|--------|----------------------|
| CM primitives | async import, oneshot future, 4-byte stream read/write, **W1 multi-chunk + 2-byte/poll backpressure** | — |
| `wasi:random` | u64 + bytes (cap 4096) | none |
| `wasi:clocks` | monotonic now/wait/resolution; **official** system `instant` `{s64,u32}` (no timezone in 0.3.0 pin) | — |
| `wasi:cli` stdio | **transitional** `future<u32>` / `stream<u8>` | W3–W4 official `result` / tuple |
| `wasi:cli/command` | **transitional** `run -> u32` | W5 official result (still not a full world) |
| `wasi:filesystem` | — | W6 Android sandbox smoke |
| `wasi:sockets` | — | W7 Android subset |
| `wasi:http` | — | W8 Android subset |
| `wasmtime-wasi` crate | not a dependency | named-only; size + thread review first |

Every remaining lane needs a **device instrument**. Details and file whitelist: the playbook.

## Ownership

Engine / linker / future/stream JNI: **this repo**. GPU: `:host-dawn` (not `wasmtime-wasi`). Pure-logic WASI: thin JNI/Kotlin stubs unless a lane explicitly evaluates `wasmtime-wasi`.

## Claims

Say: “this repo supports WASI 0.3 primitive X / package Y subset (WIT 0.3.0 pin).” Do not say “full WASI 0.3 compatible runtime.”
