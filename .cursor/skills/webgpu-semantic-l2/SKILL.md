---
name: webgpu-semantic-l2
description: >-
  Deepens hung wasi:webgpu [method]s from host-fixed stubs to described L2
  (guest fields through JNI into WasiWebGpuHost). Batches by caller resource
  then JNI family (2–4 methods per PR). Use when the user says 语义加深, 真实编组,
  L2, host-fixed, described JNI, 粗批次, or follow docs/agent/webgpu-semantic-l2.md.
---

# WebGPU semantic L2

Read and follow [`docs/agent/webgpu-semantic-l2.md`](docs/agent/webgpu-semantic-l2.md) before exploring.

1. Batch by **caller resource**, then split by **JNI family** (scalars vs buffer-borrow vs list/result). Typical PR: 2–4 methods. Do not put every method on a resource in one PR. Same verb on compute-pass / bundle-encoder is a later PR.
2. If the user listed names, keep one family. Else run `.\scripts\webgpu-semantic-l2-remaining.ps1`. If remaining host-fixed is 0, switch to skill `webgpu-midterm`. Do not re-cut create-sampler / labels / limits.
3. Local: one method → narrow test → commit; PR when the family is done.
4. Do not WebFetch WIT. Grep `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Windowed Read of `cm.rs` / `jvm.rs` / callbacks.
5. Copy the **create-buffer described** stack. `HostArg` stays Int/Long unless this PR is only bind-group / string-bearing. Use guest `rep` when non-zero; do not rebuild adapter→pass.
6. Tests: `cargo check --locked --lib` and filtered `wasi_webgpu_method -- --test-threads=1` for **all modules in the family**.
7. PR title `feat(webgpu): L2 <resource> <family> guest fields to host`, label `enhancement`.
