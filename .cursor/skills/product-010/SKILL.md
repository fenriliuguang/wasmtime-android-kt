---
name: product-010
description: >-
  After L5 productization: land the 0.1.0 product subset on Android (SPI move,
  dual-track Store, fixtures out of product linker, cli errors, outbound TCP,
  HTTP body/outbound/ctors, wasi-gfx frame loop, claim table, then publish CI).
  Use when the user says 下一刀, 0.1.0, P010, product subset, follow
  docs/agent/product-010.md, or run product-010-remaining.
---

# `0.1.0` product gates

Read and follow [`docs/agent/product-010.md`](docs/agent/product-010.md) before exploring.

1. Run `.\scripts\product-010-remaining.ps1` (or `python3 ./scripts/product-010-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** PR.
2. Order is the table in the playbook (SPI → DISC → FIX → CLIERR → TCP → HTTP → gfx → claim → publish). Needles: [`docs/scheme/product-010.md`](docs/scheme/product-010.md).
3. One lane per PR. Do **not** re-cut P0 wasi:webgpu or P1 WASI 0.3 auto knives. Never file GitHub issues on Wasmtime, WASI, wasi-webgpu, wasi-gfx, or any other upstream.
4. Do not add `wasmtime-wasi` without a size + Android thread note. Do not JS-style frame callbacks. No Central before **P010-PUB**.
5. P2 Wasmtime pin is named-only: [`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md). G-cmd / G-fs-full / listen / UDP / testsuite are not `0.1.0` gates.
6. Grep then Read ~80 lines of `cm.rs`. Hub freeze on feature lanes: no root README / `CHANGELOG.md` / `ci.yml` / `CONTRIBUTING.md` (P010-PUB may add publish workflow).
7. PR title from the playbook; docs-only label `documentation`; code `enhancement`.
