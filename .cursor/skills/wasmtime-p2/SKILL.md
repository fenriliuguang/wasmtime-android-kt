---
name: wasmtime-p2
description: >-
  After P1 WASI 0.3 close-out: keep the upstream wasmtime pin knowable,
  upgradeable, and rollback-able (patch/minor per tracking §4.1; major needs a
  short RFC). Use when the user says 下一刀, P2, Wasmtime pin, wasmtime
  tracking, follow docs/agent/wasmtime-p2.md, or run wasmtime-p2-remaining.
---

# Wasmtime pin (P2)

Read and follow [`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md) before exploring.

1. Run `.\scripts\wasmtime-p2-remaining.ps1` (or `python3 ./scripts/wasmtime-p2-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** PR.
2. Order: P2-EVAL (refresh tracking table) then P2-PATCH if the table still has `gap: p2 patch pending`. Table: [`docs/scheme/wasmtime-tracking.md`](docs/scheme/wasmtime-tracking.md).
3. One lane per PR. Do **not** re-cut P0 wasi:webgpu or P1 WASI 0.3 auto knives. Never file GitHub issues on Wasmtime, WASI, or any other upstream.
4. Do not add `wasmtime-wasi` without a size + Android thread note. Do not land major (47 → 48+) without a short upgrade RFC.
5. P1 leftover shapes (G-err / G-cmd / G-fs-full / G-sock-rest / G-http-body / G-http-ctor / G-cli-error) are named-only: [`docs/mapping/gap-wasi-p3-wit.md`](docs/mapping/gap-wasi-p3-wit.md).
6. Tests: docs-only → remaining script; crate bump → `cargo check --locked --lib` + existing `wasi_*`. Hub freeze: no root README / `CHANGELOG.md` / `ci.yml` / `CONTRIBUTING.md` on pin PRs.
7. PR title from the playbook; docs-only label `documentation`.
