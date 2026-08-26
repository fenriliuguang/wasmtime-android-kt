# Charter

**English** | [中文](charter.zh.md)

Android-first **Java/Kotlin Component runtime** on **upstream Wasmtime**.

First proposal world: canonical [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu) (WIT shape + true CM async). The runtime must not stay single-world forever: ratified WASI 0.3 packages land as guests need them.

## Vision

- Component Model as a first-class citizen (not core wasm only).  
- True CM async (`func_wrap_concurrent`, futures, later streams).  
- Bionic / ART / multi-ABI (`arm64-v8a` primary).  
- Kotlin-friendly lifecycle, threads, and errors.  
- **Citable** experimental host — see [`rfc-ecosystem-contribution.md`](rfc-ecosystem-contribution.md).

Still **experimental** until a separate product RFC. Not a compliant wasi:webgpu product by default.

## Principles

1. **Do not rewrite Dawn** (NG-7). This repo **packages and adapts** Dawn as the default GPU backend; the core runtime AAR omits Dawn so apps can supply another spec-shaped host or none. See [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md). Vendor form: [`../blocked-gpu-host.md`](../blocked-gpu-host.md).  
2. **Android-first.** Desktop is a contributor convenience.  
3. **Thin JNI.** Engine / store / linker / instance / future / resource — do not clone a full Java Wasmtime surface early.  
4. **Official async.** Never treat Latch/`sync-compat` as true CM async.  
5. **Canonical guest shape** — [`guest-shape.md`](guest-shape.md). No new host-fixed `u32` slices.  
6. **English docs are canonical** — [`../LANGUAGE.md`](../LANGUAGE.md).

## Goal stack

See [`long-term-plan.md`](long-term-plan.md): L0–L5. P0 (L3 wasi:webgpu shape) is **closed**. P1 (WASI 0.3) is **closed**. Current work is P2 (Wasmtime pin). L4 is citable host. L5 is optional productization.

## Claims

Package coordinates remain experimental (`0.x`). No default Maven Central. No production-runtime marketing.
