# Non-goals (Track B)

[中文](non-goals.md) | **English**

> Long-term revision (2026-08-11+). Authoritative detail: ZH [`non-goals.md`](non-goals.md) · [`long-term-plan.md`](long-term-plan.md).

## Absolute (until new RFC)

- Silent replace of Track A acceptance/default Demo  
- wasmtime4j as runtime dependency  
- Full Kotlin WebGPU client API rewrite  
- “Implement **all** WASI 0.3 worlds” or “pass **full** wasi-testsuite P3” as a single KPI (sliced P3 support is in-scope; see [`wasi-p3-surface.md`](wasi-p3-surface.md))  
- Compliance / “production Android Wasm runtime” marketing before a dedicated RFC  
- Default Maven Central publish  
- Second Dawn host implementation  
- Sync-compat pretending to be true CM async / WASI 0.3 async  
- Elevating wasi-gfx / multi-window to the same near-term P0 as wasi:webgpu  
- Breaking Track A’s sync-compat lock for B convenience  
- Replacing the “track upstream Wasmtime only” policy  

## Deferred

Panama desktop bind; iOS/desktop-first; full WASI cloud/CLI productization; interpreter-only fallback; monorepo merge; wasi-gfx present glue — only with evidence / RFC.

## Allowed (looks like a non-goal but is in scope)

- Ratified WASI 0.3 package **slices** (not a full suite claim)  
- **wasi:webgpu proposal** implementation & feedback (P0; ≠ compliance claim)  
- Studying 4j / `wasi-webgpu-wasmtime` as references (not runtime deps)  
