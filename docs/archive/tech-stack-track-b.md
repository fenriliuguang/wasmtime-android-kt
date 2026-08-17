# Tech stack (Track B)

[中文](tech-stack.md) | **English**

## Engine

Upstream `wasmtime` **47.x** preferred (align with Track A’s Wasmtime generation). Features: `component-model` + `component-model-async` + needed `async`. **No** wasmtime4j native.

**M2+ must use:** `func_wrap_concurrent` (or equiv), `FutureReader`/`FutureProducer`, documented store event-loop drive. Wrapping sync callbacks in `async move` alone is **not** true-async DoD.

## Binding

Rust `cdylib` → JNI (`jni` crate) → Kotlin/Java minimal API. `JNI_OnLoad` ART-safe (`JNI_VERSION_1_6` lesson from Track A). **No Panama** in phase 1.  

**Guest marshalling (2026-08-16):** canonical CM lowering per ZH RFC [`rfc-wasi-webgpu-canonical-shape.md`](rfc-wasi-webgpu-canonical-shape.md) §6 — `own`/`borrow`/record/`option`/`result`/`list`. Host-fixed u32 slices are frozen. Dawn handles may stay u32 **reps**; guests must not see bare u32 as the product return shape.

## Android

ABI: **arm64-v8a** primary, **x86_64** emulator secondary. NDK aligned with Track A scripts. Treat pointers as unsigned where TBI/PAC apply.

## Host / guests

M3+: depend on Track A `host-api` / `host-webgpu` as a **backend library** — **do not** reimplement Dawn here. Canonical guests follow `wasi:webgpu@0.3.0-rc.2` WIT; `cube-cm` is demo/legacy only.

## Non-dependencies

wasmtime4j as runtime; full wasi-http/nn as near-term gates; browser JS API.
