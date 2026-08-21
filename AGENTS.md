# Agent notes

WebGPU **leftover descriptor semantics** (blend/MSAA/cull, extra color attachments, texture view-formats, buffer mapped/label, shader hints, xr-compatible, default-queue, Dawn consume constants/required-limits): follow [`docs/agent/webgpu-guest-semantics.md`](docs/agent/webgpu-guest-semantics.md) (Cursor skill `webgpu-guest-semantics`). Next PR: `.\scripts\webgpu-guest-semantics-remaining.ps1` or `python ./scripts/webgpu-guest-semantics-remaining.py`.

Guest-pipeline P1–P5, sampler/view leftovers, pipeline-constant JNI, S1–S3 leftover JNI, canvas present, and Dawn render cite are **closed**. Do not re-cut labels / limits / `create-sampler` first-cut / canvas first-cut / P1–P5.

- WIT pin is vendored: `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Do not download it.
- Hub freeze, narrow tests, copy sources, and file whitelist are in the playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.
- **Never file upstream GitHub issues** (or Discussions used as an issue tracker) on wasi-webgpu, Wasmtime, or any other upstream. No `gh issue create`. Record Android facts only in this repo.
