# Agent notes

WebGPU product `[method]` slices: follow [`docs/agent/webgpu-shape-slice.md`](docs/agent/webgpu-shape-slice.md) (Cursor skill `webgpu-shape-slice`).

- WIT pin is vendored: `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Do not download it.
- Remaining names: `.\scripts\webgpu-shape-remaining.ps1`
- Hub freeze, narrow tests, copy sources, and file whitelist are in the playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.
