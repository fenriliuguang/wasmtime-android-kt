# Tech stack (Track B)

[中文](tech-stack.md) | **English**

## Engine

Upstream `wasmtime` **47.x** preferred (align with Track A’s Wasmtime generation). Features: `component-model` + `component-model-async` + needed `async`. **No** wasmtime4j native.

**M2+ must use:** `func_wrap_concurrent` (or equiv), `FutureReader`/`FutureProducer`, documented store event-loop drive. Wrapping sync callbacks in `async move` alone is **not** true-async DoD.

## Binding

Rust `cdylib` → JNI (`jni` crate) → Kotlin/Java minimal API. `JNI_OnLoad` ART-safe (`JNI_VERSION_1_6` lesson from Track A). **No Panama** in phase 1. Prefer explicit JNI/typed marshalling over unbounded JSON (avoid unsigned-u64 traps). Resources: **u32 rep** like Track A L2.

## Android

ABI: **arm64-v8a** primary, **x86_64** emulator secondary. NDK aligned with Track A scripts. Treat pointers as unsigned where TBI/PAC apply.

## Host / guests

M3+: depend on Track A `host-api` / `host-webgpu` / abi constants — **do not** reimplement Dawn here. Guests: M1 sync toy, M2 async smoke, M4 reuse `cube-cm` as needed.

## Non-dependencies

wasmtime4j as runtime; full wasi-http/nn as near-term gates; browser JS API.
