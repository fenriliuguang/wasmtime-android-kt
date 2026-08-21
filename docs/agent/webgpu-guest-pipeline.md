# Agent playbook: guest compute / 3D pipeline marshalling

**English** | [中文](webgpu-guest-pipeline.zh.md)

**Closed.** P1–P5 and the named first-cuts on this page are done. Current queue: [`webgpu-guest-dawn.md`](webgpu-guest-dawn.md) (`.\scripts\webgpu-guest-dawn-remaining.ps1`). Do **not** re-hang names, re-cut labels / limits / `create-sampler` first-cut, or re-cut canvas first-cut (`device`+`format`+`usage`). Leftover-descriptor F1–F9 is also closed: [`webgpu-guest-semantics.md`](webgpu-guest-semantics.md).

Pin: `wasi:webgpu@0.3.0-rc.2` at [`third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit).

Historical: guest bind-group entries, BGL lists, render-pipeline vertex buffers, render-pass depth, and texture mip/sample/dimension were marshalled into host records. Sampler/view leftovers, pipeline-constant JNI, S1–S3 leftover JNI, canvas present first-cut, and Dawn compute/render cite also landed. Remaining Dawn copies and WG-6 guest-drawn slices live on the current playbook.

Empty compute `submit` and `@builtin(vertex_index)` offscreen triangles were **not** this queue’s DoD.

## Historical lanes (do not re-cut)

| PR | Method |
|----|--------|
| P1 | `gpu-device.create-bind-group` entries |
| P2 | `gpu-device.create-bind-group-layout` all entries |
| P3 | `gpu-device.create-render-pipeline` vertex.buffers + target format |
| P4 | `gpu-command-encoder.begin-render-pass` depth + color clear |
| P5 | `gpu-device.create-texture` mip / sample-count / dimension |
| Named | Sampler / view leftovers; pipeline-constant map resource; S1–S3 leftover JNI; canvas present first-cut; Dawn render cite |

User prompt that works now: “follow `docs/agent/webgpu-guest-dawn.md`”.
