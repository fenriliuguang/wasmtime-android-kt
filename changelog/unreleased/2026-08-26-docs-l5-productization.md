### Docs — accept L5 productization and wasi-gfx frame-loop RFCs (2026-08-26)

- Accept [`docs/scheme/rfc-l5-productization.md`](../../docs/scheme/rfc-l5-productization.md): product class B; perpetual `0.x`; WASI **product subset**; dual-track GPU attach; move `ExperimentalHostCallbacks` out of `runtime` public SPI before `0.1.0`; **no Central / `0.0.x-preview` until `0.1.0` gates**
- Accept intent [`docs/scheme/rfc-wasi-gfx-frame-loop.md`](../../docs/scheme/rfc-wasi-gfx-frame-loop.md): `0.1.0` present loop via wasi-gfx `on-frame` stream (not a P0 re-cut)
- Align charter, non-goals (NG-6 / NG-9 / DG-6), API stability, indexes, ecosystem / GPU RFCs, gap table (`0.1.0` backlog still not `wasmtime-p2-remaining` `Next:`)
- No crate bump; no SPI move in this PR; publishing CI still wait-for-gates
