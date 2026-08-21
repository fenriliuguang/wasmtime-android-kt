---
name: webgpu-midterm
description: >-
  After default wasi:webgpu shape remaining is 0 (canvas omitted) and default
  semantic-L2 host-fixed remaining is 0: WG-6 gpu-canvas-context, S1–S3 guest
  descriptors/reps, record-* maps (only if named), or a citable Dawn/WG-5 slice.
  Use when the user says 下一阶段, midterm, WG-6, canvas, S1, S3, request-adapter,
  request-device, device.queue, record-*, 真机, 可引用, WG-5, or follow
  docs/agent/webgpu-midterm.md.
---

# WebGPU midterm lanes

Read and follow [`docs/agent/webgpu-midterm.md`](docs/agent/webgpu-midterm.md) before exploring.

1. Run `.\scripts\webgpu-midterm-remaining.ps1` (or `python3 ./scripts/webgpu-midterm-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** PR.
2. Order: A1 canvas shape → A2 configure L2 → A3 get-current-texture (+ unconfigure) → A4 get-configuration → B1 queue `device.rep` → B2 request-device → B3 request-adapter. Lane C (`record-*`) and D (Dawn cite / WG-5) only if the user named them (`-IncludeRecords` for C).
3. One lane per PR. Do not mix shape hang with L2. Do not re-cut labels / limits / create-sampler. Never file GitHub issues on wasi-webgpu, Wasmtime, or any other upstream.
4. Do not WebFetch WIT. Grep `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Windowed Read of `cm.rs` / `jvm.rs` (~80 lines). Copy the playbook stack for that lane only.
5. Canvas: test-only `get-canvas-context`; never product `surface-*` or wasi-gfx. S1–S3: keep true async; B1 must not rebuild adapter→device.
6. Tests: `cargo check --locked --lib` + filtered `wasi_webgpu_method -- --test-threads=1`. Device instruments only for Lane D.
7. PR title from the playbook; label `enhancement` (D docs-only: `documentation`).
