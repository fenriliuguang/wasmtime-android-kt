# `0.1.0` product queue (tracking)

**English** | [中文](product-010.zh.md)

Living **auto** queue for L5 `0.1.0` gates. Playbook: [`../agent/product-010.md`](../agent/product-010.md). Policy: [`rfc-l5-productization.md`](rfc-l5-productization.md). Gfx shape: [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md).

P0 / P1 auto knives stay **closed**. P2 Wasmtime pin is a **named** queue ([`wasmtime-p2.md`](../agent/wasmtime-p2.md)) — not this script’s `Next:`.

Remaining: `python3 ./scripts/product-010-remaining.py`. A lane drops when its **`gap: p010 … pending`** needle leaves **this file**. Do **not** remove a needle without landing that lane’s DoD.

## Needles (auto order)

<!-- remaining.py greps these exact strings. Keep one per unfinished lane. -->

| Lane | Needle (delete when landed) |
|------|-----------------------------|
| P010-SPI | landed 2026-08-27 |
| P010-DISC | landed 2026-08-27 |
| P010-FIX | landed 2026-08-27 |
| P010-CLIERR | landed 2026-08-27 |
| P010-TCP | landed 2026-08-27 |
| P010-HBODY | landed 2026-08-27 |
| P010-HOUT | landed 2026-08-27 |
| P010-HCTOR | landed 2026-08-27 |
| P010-GFXP | landed 2026-08-27 |
| P010-GFXH | landed 2026-08-27 |
| P010-GFXL | landed 2026-08-27 (skeleton: two pre-buffered frames) |
| P010-GFXB | landed 2026-08-27 (product request-adapter/request-device) |
| P010-GFXV | gap: p010 gfx-vsync pending |
| P010-DEMO | gap: p010 demo pending |
| P010-CLAIM | landed 2026-08-27 |
| P010-PUB | landed 2026-08-27 |

## Named-only (never `Next:`)

G-cmd full world, G-fs-full, listen/UDP, wasi-testsuite, `wasmtime-wasi` crate, this-repo **1.0.0**, CTS, P0/P1 re-cuts, Wasmtime **major**. Filesystem Smoke already stands (L5).
