# Agent notes

WebGPU **guest compute / 3D pipeline** (bind-group entries, BGL lists, vertex layouts, depth pass, texture mip/sample/dimension): follow [`docs/agent/webgpu-guest-pipeline.md`](docs/agent/webgpu-guest-pipeline.md) (Cursor skill `webgpu-guest-pipeline`). Next PR: `.\scripts\webgpu-guest-pipeline-remaining.ps1` or `python ./scripts/webgpu-guest-pipeline-remaining.py`.

Shape hang, default described L2, and midterm first-cuts are **closed**. Do not re-cut labels / limits / `create-sampler` first-cut / canvas first-cut.

- WIT pin is vendored: `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Do not download it.
- Hub freeze, narrow tests, copy sources, and file whitelist are in the playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.
- **Never file upstream GitHub issues** (or Discussions used as an issue tracker) on wasi-webgpu, Wasmtime, or any other upstream. No `gh issue create`. Record Android facts only in this repo.
