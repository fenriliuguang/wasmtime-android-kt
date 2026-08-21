---
name: webgpu-guest-pipeline
description: >-
  After wasi:webgpu shape hang, default described L2, and midterm first-cuts:
  marshall guest bind-group entries, BGL lists, render-pipeline vertex buffers,
  render-pass depth, and texture mip/sample/dimension through JNI into
  DawnWasiWebGpuHost. Use when the user says 下一刀, guest pipeline, bind-group,
  3D, compute, P1–P5, 上屏, canvas present, or follow
  docs/agent/webgpu-guest-pipeline.md. Also use for leftover 语义加深 / 下一阶段
  once the old shape / L2 / midterm queues are closed.
---

# WebGPU guest pipeline

Read and follow [`docs/agent/webgpu-guest-pipeline.md`](docs/agent/webgpu-guest-pipeline.md) before exploring.

1. Run `.\scripts\webgpu-guest-pipeline-remaining.ps1` (or `python3 ./scripts/webgpu-guest-pipeline-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** PR.
2. Order: P1 bind-group `entries` → P2 BGL all entries → P3 render-pipeline `vertex.buffers` + guest color format → P4 begin-render-pass depth + clear → P5 create-texture mip/sample/dimension. Canvas present, sampler/view leftovers, pipeline constants, S1–S3 leftovers, and Dawn render cite only if the user named them.
3. One lane per PR. Do not re-hang `[method]` names. Do not re-cut labels / limits / create-sampler first-cut / canvas first-cut. Never file GitHub issues on wasi-webgpu, Wasmtime, or any other upstream.
4. Do not WebFetch WIT. Grep `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Windowed Read of `cm.rs` / `jvm.rs` (~80 lines). Copy the playbook stack. Forward into existing `WasiWebGpuHost` records — Dawn host already accepts them.
5. No product `surface-*`. No wasi-gfx. No CTS claim. `request-adapter` with no backend → guest `none`. Keep true async.
6. Tests: `cargo check --locked --lib` + filtered `wasi_webgpu_method -- --test-threads=1`. Device instruments only for a named cite / on-screen PR.
7. PR title from the playbook; label `enhancement` (docs-only cite: `documentation`).
