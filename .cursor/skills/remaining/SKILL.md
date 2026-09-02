---
name: remaining
description: >-
  Living close-out after native Dawn host: Dawn C full bind of remaining
  pin methods, wasi-gfx surface size/resize, then remaining pin input
  streams. Use when the user says 下一刀, remaining, BIND, GFX-SIZE,
  GFX-PIN, Dawn C full bind, or remaining.py.
---

# Remaining close-out

Read [`docs/agent/remaining.md`](docs/agent/remaining.md) before exploring.

1. Run `python3 ./scripts/remaining.py`. Do **only** the printed **Next:** as **one commit**.
2. Order: **BIND** → **GFX-SIZE** → **GFX-PIN**. Needles: [`docs/scheme/remaining.md`](docs/scheme/remaining.md).
3. BIND is full-pin Dawn C (`webgpu.h` when `.so` is loaded), not a cube subset. GFX-SIZE is `height` / `width` / `request-set-size` / `on-resize`. GFX-PIN is `on-pointer-*` / `on-key-*`.
4. Named-only (never `Next:`): `context.unconfigure`, timestamped `frame-event`, Lost/Outdated `result`, multi-window, P2 Wasmtime, G-cmd, G-fs-full, listen/UDP, CTS, `wasmtime-wasi`, this-repo 1.0.
5. Reuse `cm.rs` lowering. Grep then Read ~80 lines. Do not reimplement `exp_*` JNI. Do not add `wasmtime-wasi` without a size + Android thread note.
6. Coordinate stays **`0.1.0`** until that release is pressed. Never file upstream GitHub issues.
