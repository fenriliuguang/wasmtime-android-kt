# Agent playbook: leftover descriptor semantics

> **Archived 2026-08-22.** Do not implement from this file. P0 close-out: [`p0-wasi-webgpu.md`](p0-wasi-webgpu.md). Current queue: [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md).

**English** | [中文](webgpu-guest-semantics.zh.md)

**Closed.** F1–F9 and the named handle-0 / full `required-features` lanes on this page are done. Current queue: [`webgpu-guest-dawn.md`](webgpu-guest-dawn.md) (`.\scripts\webgpu-guest-dawn-remaining.ps1`). Do **not** re-hang `[method]` names, re-cut P1–P5 / F1–F9, re-cut labels / limits / `create-sampler` first-cut, or re-cut canvas first-cut.

Pin: `wasi:webgpu@0.3.0-rc.2` at [`third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit).

Historical: leftover optional WIT descriptor fields were marshalled into Kotlin records; Dawn consume of pipeline constants and required-limits landed as F8–F9. Remaining Dawn copies, write-mask / stencil / canvas configuration extras, `layout: auto`, and WG-6 guest-drawn slices live on the current playbook.

Empty compute `submit` and `@builtin(vertex_index)` offscreen triangles were **not** this queue’s DoD.

## Historical lanes (do not re-cut)

| PR | Method |
|----|--------|
| F1 | `gpu-device.create-render-pipeline` blend / MSAA / cull / strip (JNI + Kotlin record) |
| F2 | `gpu-command-encoder.begin-render-pass` all color attachments |
| F3 | `gpu-device.create-texture` view-formats + label |
| F4 | `gpu-device.create-buffer` mapped-at-creation + label |
| F5 | `gpu-device.create-shader-module` label + compilation-hints |
| F6 | `gpu.request-adapter` xr-compatible |
| F7 | `gpu-adapter.request-device` default-queue label |
| F8 | Dawn consume pipeline constants |
| F9 | Dawn consume required-limits |
| Named | SupportedLimits handle-0; full `required-features` list |

User prompt that works now: “follow `docs/agent/webgpu-guest-dawn.md`”.
