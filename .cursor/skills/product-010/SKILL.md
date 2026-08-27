---
name: product-010
description: >-
  After L5 productization: land the 0.1.0 product subset on Android (SPI move,
  dual-track Store, fixtures out of product linker, cli errors, outbound TCP,
  HTTP body/outbound/ctors, wasi-gfx frame loop (product adapter/device +
  vsync on-frame), out-of-tree demo README link + device row, claim table,
  then publish CI). Remaining after PUB: P010-GFXB then P010-GFXV then
  P010-DEMO.
  Use when the user says 下一刀, 0.1.0, P010, product subset, follow
  docs/agent/product-010.md, or run product-010-remaining.
---

# `0.1.0` product gates

Read and follow [`docs/agent/product-010.md`](docs/agent/product-010.md) before exploring.

1. Run `.\scripts\product-010-remaining.ps1` (or `python3 ./scripts/product-010-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** PR.
2. Order is the table in the playbook (SPI → DISC → FIX → CLIERR → TCP → HTTP → gfx pin/host/loop → **GFXB → GFXV → DEMO** → claim → publish). Needles: [`docs/scheme/product-010.md`](docs/scheme/product-010.md). After PUB, remaining **Next:** is **P010-GFXB** until the complete loop and demo link land.
3. One lane per PR. Do **not** re-cut P0 wasi:webgpu or P1 WASI 0.3 auto knives. Never file GitHub issues on Wasmtime, WASI, wasi-webgpu, wasi-gfx, or any other upstream.
4. Do not add `wasmtime-wasi` without a size + Android thread note. Do not JS-style frame callbacks. Publishing CI is **P010-PUB** (already landed); do not press when secrets are missing. Complete frame loop is **P010-GFXB** then **P010-GFXV**. Last: **P010-DEMO** (README out-of-tree demo link + named device row). Do not vendor the demo into this repo.
5. P2 Wasmtime pin is named-only: [`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md). G-cmd / G-fs-full / listen / UDP / testsuite are not `0.1.0` gates.
6. Grep then Read ~80 lines of `cm.rs`. Hub freeze on feature lanes: no root README / `CHANGELOG.md` / `ci.yml` / `CONTRIBUTING.md`. Exceptions: this playbook / gate-amendment PR; **P010-PUB** (publish workflow); **P010-DEMO** (README Demo section only — one out-of-tree link; do not vendor the demo).
7. PR title from the playbook; docs-only label `documentation`; code `enhancement`.
