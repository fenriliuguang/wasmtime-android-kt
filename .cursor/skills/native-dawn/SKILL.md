---
name: native-dawn
description: >-
  After 0.1.0: full-pin wasi:webgpu consume via in-process Dawn C (dispatch,
  one C-API .so, host table, boot/resources/pipelines/encoders/queue, remaining
  method suite, ANativeWindow surface, default switch, claim, device instruments).
  Cube is demo evidence only. Use when the user says 下一刀, native-dawn, ND-DISP,
  Dawn C, bypass androidx JNI, or run native-dawn-remaining.
---

# Native Dawn host (full pin)

Read and follow [`docs/agent/native-dawn.md`](docs/agent/native-dawn.md) before exploring.

1. Run `.\scripts\native-dawn-remaining.ps1` (or `python3 ./scripts/native-dawn-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** PR.
2. Order is the playbook table (DISP → SO → HOST → BOOT → RES → PIPE → ENC → QUEUE → **REST** → SURF → DEFAULT → CLAIM → DEVICE). Needles: [`docs/scheme/native-dawn.md`](docs/scheme/native-dawn.md). **ND-REST** is the full-pin gate — do not close it with a cube demo.
3. One lane per PR. Do **not** re-cut P0 G1–G9 / F1–F9 / WG-6 or P1 WASI auto knives. Never file GitHub issues on Wasmtime, WASI, Dawn, androidx, or any other upstream.
4. Reuse `cm.rs` lowering, `fixtures/w1`, `wasi_webgpu_method`, hitch invariants, `DawnWasiWebGpuHost.kt` as a **mapping spec**. Do not reimplement `exp_*` JNI in Rust. Do not grow `WebGpuBackend` into a Kotlin WebGPU client.
5. One Dawn `.so` with C API. Changelog size + Android thread on **ND-SO**. Do not ship androidx bundled + self-built together as default. Do not JS-style frame callbacks.
6. P2 Wasmtime pin is named-only: [`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md). `0.1.0` queue is empty: [`docs/agent/product-010.md`](docs/agent/product-010.md).
7. Grep then Read ~80 lines of `cm.rs`. Hub freeze on feature lanes: no root README / `CHANGELOG.md` / `ci.yml` / `CONTRIBUTING.md` except this playbook / gate-amendment PR.
8. PR title from the playbook; docs-only label `documentation`; code `enhancement`.
