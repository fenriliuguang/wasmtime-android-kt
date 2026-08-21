# Agent notes

WebGPU product `[method]` **shape hangs**: follow [`docs/agent/webgpu-shape-slice.md`](docs/agent/webgpu-shape-slice.md) (Cursor skill `webgpu-shape-slice`). Remaining names: `.\scripts\webgpu-shape-remaining.ps1`. Canvas is **not** this queue.

WebGPU **semantic L2** (guest fields → JNI → `WasiWebGpuHost`): follow [`docs/agent/webgpu-semantic-l2.md`](docs/agent/webgpu-semantic-l2.md) (Cursor skill `webgpu-semantic-l2`). Remaining: `.\scripts\webgpu-semantic-l2-remaining.ps1`. Batch by caller resource, then one JNI family per PR. Skip S1–S3 / canvas / `record-*` here.

WebGPU **midterm** (after those two remaining lists are empty aside from omitted canvas/records): follow [`docs/agent/webgpu-midterm.md`](docs/agent/webgpu-midterm.md) (Cursor skill `webgpu-midterm`). Next PR: `.\scripts\webgpu-midterm-remaining.ps1` or `python3 ./scripts/webgpu-midterm-remaining.py`. Lanes: WG-6 `gpu-canvas-context`, S1–S3 guest reps/descriptors, named `record-*`, or a citable Dawn/WG-5 slice.

- WIT pin is vendored: `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Do not download it.
- Hub freeze, narrow tests, copy sources, and file whitelist are in the playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.
- **Never file upstream GitHub issues** (or Discussions used as an issue tracker) on wasi-webgpu, Wasmtime, or any other upstream. No `gh issue create`. Record Android facts only in this repo.
