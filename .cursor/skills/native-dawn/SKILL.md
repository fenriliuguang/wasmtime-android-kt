---
name: native-dawn
description: >-
  CLOSED consume leftover (ND-DEVICE / #299). Full-pin wasi:webgpu via Dawn C
  already landed. Living leftover is empty. Use when the user names
  native-dawn, ND-DISP, ND-REST, or native-dawn-remaining.
---

# Native Dawn host (full pin) — consume empty

Consume leftover is **empty**. Read [`docs/agent/native-dawn.md`](docs/agent/native-dawn.md) only if the user named an `ND-*` leftover.

1. `python3 ./scripts/native-dawn-remaining.py` prints empty. Do **not** invent new `ND-*` needles. Do **not** open a consume PR.
2. Cube is demo evidence only — never consume DoD.
3. Do **not** re-cut P0 G1–G9 / F1–F9 / WG-6 or P1 WASI auto knives. Never file GitHub issues on Wasmtime, WASI, Dawn, androidx, or any other upstream.
4. Reuse `cm.rs` lowering; do not reimplement `exp_*` JNI. Grep then Read ~80 lines of `cm.rs`.
5. P2 Wasmtime pin is named-only: [`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md). `0.1.0` queue is empty: [`docs/agent/product-010.md`](docs/agent/product-010.md).
