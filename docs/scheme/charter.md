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

**L5 accepted** ([`rfc-l5-productization.md`](rfc-l5-productization.md)): product class **B**, perpetual **`0.x`**, Maven Central only after **`0.1.0` gates**. Still **not** a compliant wasi:webgpu product (NG-5). Frame loop: [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md).

## Principles

1. **Do not rewrite Dawn** (NG-7). This repo **packages and adapts** Dawn as the default GPU backend; the core runtime AAR omits Dawn so apps can supply another spec-shaped host or none. See [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md). Vendor form: [`../blocked-gpu-host.md`](../blocked-gpu-host.md).  
2. **Android-first.** Desktop is a contributor convenience.  
3. **Thin JNI.** Engine / store / linker / instance / future / resource — do not clone a full Java Wasmtime surface early.  
4. **Official async.** Never treat Latch/`sync-compat` as true CM async.  
5. **Canonical guest shape** — [`guest-shape.md`](guest-shape.md). No new host-fixed `u32` slices.  
6. **English docs are canonical** — [`../LANGUAGE.md`](../LANGUAGE.md).

## Goal stack

See [`long-term-plan.md`](long-term-plan.md): L0–L5. P0 (L3 wasi:webgpu shape) is **closed**. P1 (WASI 0.3) is **closed**. **`0.1.0` remaining** is the complete gfx frame loop (**P010-GFXB** → **P010-GFXV**) then **P010-DEMO** (README out-of-tree demo + named device row) ([`../agent/product-010.md`](../agent/product-010.md)). P2 Wasmtime pin is **named**. L4 is citable host. **L5 is accepted** (0.x product subset; coordinates `0.1.0`).

## Claims

Package coordinates stay **`0.x`** until upstream 1.0 gates in L5 §6. **`0.1.0`** publishing CI is [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml) (same GAV on Central + GitHub Packages). Do not press when secrets are missing. No `0.0.x-preview` Central. No CTS / WASI-1.0 distro marketing.
