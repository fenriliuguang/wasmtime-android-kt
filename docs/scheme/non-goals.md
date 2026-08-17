# Non-goals

**English** | [中文](non-goals.zh.md)

Hard boundaries until a new RFC.

| ID | Do not |
|----|--------|
| NG-2 | Depend on **wasmtime4j** as the runtime (including transitive CM execution) |
| NG-3 | Reimplement a full **Kotlin WebGPU client API** (engine-shaped) |
| NG-4 | Treat “all WASI 0.3 worlds” or “full wasi-testsuite P3” as the single KPI |
| NG-5 | Claim a **compliant wasi:webgpu product** or production Android Wasm runtime before a dedicated RFC |
| NG-6 | Default **Maven Central** publish |
| NG-7 | Implement a **second Dawn renderer** in this repo (packaging / adapting one Dawn as `:host-dawn` is allowed; see [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md)) |
| NG-8 | Treat Latch / sync-compat as **true** CM async / WASI 0.3 async DoD |
| NG-9 | Promote **wasi-gfx / multi-window** to the same near-term P0 as `wasi:webgpu` |
| NG-11 | Replace “track upstream Wasmtime” with a non-official engine |
| NG-12 | Accept **host-fixed descriptor + transitional u32** as the DoD for **new** wasi:webgpu slices |

Removed from the living table (historical dual-product policy): silent-replace of an external demo runtime; requiring that demo to break sync-compat. Those files live in [`../archive/`](../archive/README.md).

## Deferred (separate RFC)

| ID | Item |
|----|------|
| DG-1 | Panama desktop bindings |
| DG-2 | iOS / desktop as first-class |
| DG-3 | Full cloud/CLI WASI distro |
| DG-4 | Interpreter fallback (no Cranelift) |
| DG-6 | Minimal wasi-gfx present glue |

## Allowed

- Ratified WASI 0.3 **slices** per [`wasi-p3-surface.md`](wasi-p3-surface.md)  
- Implementing and feeding back on the **wasi:webgpu proposal** (not a compliance claim)  
- Comparing with `wasi-webgpu-wasmtime` and other hosts  
- A **pluggable** GPU backend — default Dawn bundle; core without Dawn ([`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md)). Unpublished artifacts: [`../blocked-gpu-host.md`](../blocked-gpu-host.md)
