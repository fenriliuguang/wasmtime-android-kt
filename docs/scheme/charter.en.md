# Charter: wasmtime-android-kt (Track B)

[中文](charter.md) | **English**

> **Status: short-term M0–M5 archived; long-term plan current (2026-08-11).** Authoritative ZH: [`charter.md`](charter.md) · [`long-term-plan.md`](long-term-plan.md).  
> Sister Track A: [`wasi-webgpu-jvm-mvp`](../../../wasi-webgpu-jvm-mvp) — **locked sync-compat**.  
> Index: [`README.en.md`](README.en.md) · [`dual-track.en.md`](dual-track.en.md) · [`milestones.en.md`](milestones.en.md) · [`non-goals.en.md`](non-goals.en.md)

## 1. Background

Track A already ships L2 (`WasiWebGpuHost` / Dawn), L1 via **wasmtime4j** + patches, and a stable CM cube device baseline. Standard wasi:webgpu 0.3 uses WIT **`async func`**; Track A runs **sync-compat** (block in host callbacks).

Track A’s true-CM-async spike showed Cargo `component-model-async` is present, but **Java has no future create/write/complete/reject API**. Gate closed: see Track A [`archive-true-cm-async-dod.en.md`](../../../wasi-webgpu-jvm-mvp/docs/scheme/archive-true-cm-async-dod.en.md).

**Why Track B:** the gap is the binding, not Android’s ability to run upstream Wasmtime CM async. Track A must stay demoable; upstream Wasmtime already exposes `FutureProducer` / `FutureReader` / `func_wrap_concurrent`. Long-term we want an **Android-first JVM Wasm runtime**, not only a webgpu patch.

Metaphor (from Track A): build the **wiring (L2)** first; Track B is a new **socket (L1)**.

## 2. Vision & goal stack

**Long-term:** Java/Kotlin Wasm runtime specialized for Android — Component Model first-class, true CM async, Bionic/ART/multi-ABI, Kotlin-friendly lifecycle/threading/errors. First scenario remains webgpu host glue; runtime must not be forever single-world.

**Near-term:** thin L1 over **upstream `wasmtime`**, custom JNI + minimal Kotlin/Java API (no wasmtime4j), instantiate/link/call, at least one true CM async host import, Android smoke against the **same L2**.

```text
L0  build skeleton (Gradle/NDK/Rust)     → M0
L1a sync CM min loop                     → M1
L1b true CM async smoke                  → M2 (hard gate)
L1c plug Track A L2                      → M3
L1d Android on-screen smoke              → M4
L2* runtime productization               → M5+
```

## 3. Principles

1. L2 must not depend on L1.  
2. Android-first.  
3. Thin binding; don’t clone full wasmtime4j surface early.  
4. Official async semantics — no sync-compat as “true async” DoD.  
5. Isolate git/CI from Track A; A stays sync-compat-locked.  
6. Reuse Track A **lessons** (JNI_VERSION, TBI, rep↔u32); do **not** depend on 4j `.so`.  
7. Honest threading — see [`threading-android.en.md`](../mapping/threading-android.en.md).

## 4. Dependencies (summary)

Upstream `wasmtime` 47.x (align with Track A where practical); Rust cdylib + JNI; no Panama in phase 1; consume Track A `host-api` / `host-webgpu` at M3+; guests: new async smoke, later reuse `cube-cm`. Details: [`tech-stack.en.md`](tech-stack.en.md).

## 5. Milestones (summary)

M0 load `.so` → M1 sync CM → **M2 true async gate** → M3 L2 → M4 on-screen → M5 harden. Full DoD: [`milestones.en.md`](milestones.en.md).

## 6. Claims

Remain **experimental**; no compliant wasi:webgpu product claim; no default external publish.

## 7. This init deliverable

Repo + planning docs only. **No** Gradle/native code in this drop.
