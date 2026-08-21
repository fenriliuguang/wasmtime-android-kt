---
name: webgpu-guest-semantics
description: >-
  After guest-pipeline P1–P5 and named first-cuts: marshall leftover optional
  WIT descriptor fields (blend/MSAA/cull, extra color attachments, texture
  view-formats, buffer mapped/label, shader hints, xr-compatible, default-queue)
  and have Dawn consume already-snapshotted pipeline constants and required-limits.
  Use when the user says 下一刀, 语义加深, leftover descriptor, F1–F9, blend, MRT,
  view-formats, mapped-at-creation, xr-compatible, default-queue, Dawn constants,
  required-limits consume, or follow docs/agent/webgpu-guest-semantics.md.
---

# WebGPU leftover descriptor semantics

Read and follow [`docs/agent/webgpu-guest-semantics.md`](docs/agent/webgpu-guest-semantics.md) before exploring.

1. Run `.\scripts\webgpu-guest-semantics-remaining.ps1` (or `python3 ./scripts/webgpu-guest-semantics-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** PR.
2. Order: F1 render-pipeline blend/MSAA/cull/strip → F2 all color attachments → F3 texture view-formats/label → F4 buffer mapped/label → F5 shader hints/label → F6 xr-compatible → F7 default-queue → F8 Dawn consume pipeline constants → F9 Dawn consume required-limits. SupportedLimits handle-0 and full required-features only if the user named them.
3. One lane per PR. Do not re-hang `[method]` names. Do not re-cut P1–P5, labels/limits/sampler/canvas first-cuts, the constants map resource, or S1–S3 JNI. Never file GitHub issues on wasi-webgpu, Wasmtime, or any other upstream.
4. Do not WebFetch WIT. Grep `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Windowed Read of `cm.rs` / `jvm.rs` (~80 lines). Copy the playbook stack. Forward into existing `WasiWebGpuHost` records — extend them if a field is missing. No new `HostArg` variants.
5. No product `surface-*`. No wasi-gfx. No CTS claim. `request-adapter` with no backend → guest `none`. Keep true async.
6. Tests: `cargo check --locked --lib` + filtered `wasi_webgpu_method -- --test-threads=1`. Device instruments only for the named handle-0 PR.
7. PR title from the playbook; label `enhancement` (handle-0: `bug`).
