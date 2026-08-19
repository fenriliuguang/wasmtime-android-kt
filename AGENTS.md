# Agent notes

WebGPU product `[method]` **shape hangs**: follow [`docs/agent/webgpu-shape-slice.md`](docs/agent/webgpu-shape-slice.md) (Cursor skill `webgpu-shape-slice`). Remaining names: `.\scripts\webgpu-shape-remaining.ps1`.

WebGPU **semantic L2** (guest fields → JNI → `WasiWebGpuHost`): follow [`docs/agent/webgpu-semantic-l2.md`](docs/agent/webgpu-semantic-l2.md) (Cursor skill `webgpu-semantic-l2`). Remaining: `.\scripts\webgpu-semantic-l2-remaining.ps1`. One `[method]` per PR.

- WIT pin is vendored: `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Do not download it.
- Hub freeze, narrow tests, copy sources, and file whitelist are in the playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.
