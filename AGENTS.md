# Agent notes

WebGPU **Dawn consume + WG-6 leftovers** (copy already-snapshotted blend/cull/MSAA, texture view-formats, shader hints, xr-compatible, default-queue into androidx.webgpu; marshall write-mask / depth-stencil leftovers / canvas configuration extras; accept pipeline `layout: auto`; named guest-drawn compute/3D/present): follow [`docs/agent/webgpu-guest-dawn.md`](docs/agent/webgpu-guest-dawn.md) (Cursor skill `webgpu-guest-dawn`). Next PR: `.\scripts\webgpu-guest-dawn-remaining.ps1` or `python ./scripts/webgpu-guest-dawn-remaining.py`.

Guest-pipeline P1–P5, leftover-descriptor F1–F9, handle-0, required-features full list, sampler/view leftovers, pipeline-constant JNI, S1–S3 leftover JNI, canvas present first-cut, and Dawn render/compute cite are **closed**. Do not re-cut labels / limits / `create-sampler` first-cut / canvas first-cut / P1–P5 / F1–F9.

- WIT pin is vendored: `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Do not download it.
- Hub freeze, narrow tests, copy sources, and file whitelist are in the playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.
- **Never file upstream GitHub issues** (or Discussions used as an issue tracker) on wasi-webgpu, Wasmtime, or any other upstream. No `gh issue create`. Record Android facts only in this repo.
