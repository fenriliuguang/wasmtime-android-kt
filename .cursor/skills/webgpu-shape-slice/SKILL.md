---
name: webgpu-shape-slice
description: >-
  Runs a wasi:webgpu S6+ product [method] shape slice using the repo playbook
  (vendored WIT, copy-one-template, hub freeze, narrow tests, PR). Use when the
  user says 下一刀, S6+, hang [method], shape slice, webgpu remaining methods,
  or asks to advance the wasi:webgpu product surface.
---

# WebGPU shape slice

Read and follow [`docs/agent/webgpu-shape-slice.md`](docs/agent/webgpu-shape-slice.md) before exploring.

1. If the user listed `[method]` names, that is the cut. Else run `.\scripts\webgpu-shape-remaining.ps1`.
2. Do not WebFetch WIT. Grep `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`.
3. Do not read `cm.rs` / `webgpu_abi.rs` / `ExperimentalWebGpuBridge.kt` whole; Grep then windowed Read.
4. Copy **one** native twin from the playbook table. Duplicate ABI types in the test (no `webgpu_abi` import).
5. DoD: changelog fragment, fixture table row + parse/validate lines (not the Transitional paragraph), Kotlin instrument, `enhancement` PR.
6. Tests: `cargo check --locked --lib` and filtered `wasi_webgpu_method` with `--test-threads=1`.
