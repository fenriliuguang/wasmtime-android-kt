# Agent notes

**P1 WASI 0.3** (official package WIT on Android + a device instrument per lane): follow [`docs/agent/wasi-p3.md`](docs/agent/wasi-p3.md) (Cursor skill `wasi-p3`). Next PR: `.\scripts\wasi-p3-remaining.ps1` or `python ./scripts/wasi-p3-remaining.py`.

P0 `wasi:webgpu` is **closed**. Do not re-cut guest-pipeline P1–P5, leftover F1–F9, Dawn consume G1–G9, or WG-6. Close-out: [`docs/archive/p0-wasi-webgpu.md`](docs/archive/p0-wasi-webgpu.md). WIT ↔ androidx holes: [`docs/mapping/gap-webgpu-wit-androidx.md`](docs/mapping/gap-webgpu-wit-androidx.md).

- Hub freeze, narrow tests, and file whitelist are in the P1 playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.
- **Never file upstream GitHub issues** (or Discussions used as an issue tracker) on WASI, Wasmtime, wasi-webgpu, or any other upstream. No `gh issue create`. Record Android facts only in this repo.
