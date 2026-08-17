# Boundary with Track A (Track A = demo)

[中文](dual-track.md) | **English**

> **2026-08-16:** Parallel product tracks are over. Authoritative RFC (ZH): [`rfc-wasi-webgpu-canonical-shape.md`](rfc-wasi-webgpu-canonical-shape.md).  
> This page is a short EN pointer; ZH is source of truth.

## Roles

| Track | Role |
|-------|------|
| **A** `wasi-webgpu-jvm-mvp` | **Simple demo**: experimental CM cube + wasmtime4j + locked sync-compat. **Not** this repo’s ABI upstream. |
| **B** (this repo) | Android-first JVM Component runtime. **Sole** place that advances canonical `wasi:webgpu` guest shape on official Wasmtime. |

Do **not** schedule “A changes flat L2 first, B follows.”

## Still true

- Do not silently replace Track A’s default Demo runtime (NG-1).  
- Do not break Track A’s sync-compat lock for this repo’s convenience (NG-10).  
- Do not depend on wasmtime4j here.  
- Dawn Host may be consumed as a **library**; **WIT marshalling is owned here**.

## Talk track

- Demo cube → Track A.  
- Canonical wasi:webgpu + WASI 0.3 on Android JVM → this repo.  
- Host-fixed u32 `[method]` slices are frozen; next code slice is S1 in the RFC.
