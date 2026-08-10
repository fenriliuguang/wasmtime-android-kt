# Dual-track contract (A ↔ B)

[中文](dual-track.md) | **English**

## Roles

| Track | Path | Role |
|-------|------|------|
| **A** | `wasi-webgpu-jvm-mvp` | wasi:webgpu Host MVP; L1=wasmtime4j; **async locked sync-compat** |
| **B** | `wasmtime-android-kt` | Android-first JVM Wasm runtime; L1=upstream Wasmtime thin bind; true CM async goal |

## Track A lock (from 2026-08-10)

1. Keep **sync-compat** on default / acceptance paths.  
2. **Do not** rework Dawn await path / Linker futures / move instrumentation to async Guest for “true CM async”.  
3. Gate archive remains authoritative.  
4. Track A may still do stability/docs/non-async work.  
5. Switching L1 to Track B requires a separate RFC + dual green — never silent.

## Share vs isolate

**Share:** L2 artifacts, ABI constants, guest bytes (read-only), mapping docs, patch *lessons*.  
**Isolate:** no wasmtime4j as Track B runtime; no forcing Track A CI to build B; no replacing A’s default Demo; no sync-compat as B’s M2 DoD.

## Integration

M1–M3: B self-smoke; depend on A engineered Maven locally if needed.  
M4: optional A Demo entry behind flag; separate instrumented cases; **do not** replace A’s primary script.  
Later: RFC with rollback before flipping acceptance.
