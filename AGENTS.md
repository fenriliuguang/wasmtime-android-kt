# Agent notes

**P1 WASI 0.3** (official package WIT on Android + a device instrument per lane): follow [`docs/agent/wasi-p3.md`](docs/agent/wasi-p3.md) (Cursor skill `wasi-p3`). Next PR: `.\scripts\wasi-p3-remaining.ps1` or `python ./scripts/wasi-p3-remaining.py`.

P0 `wasi:webgpu` is **closed**. Do not re-cut guest-pipeline P1–P5, leftover F1–F9, Dawn consume G1–G9, or WG-6. Close-out: [`docs/archive/p0-wasi-webgpu.md`](docs/archive/p0-wasi-webgpu.md). WIT ↔ androidx holes: [`docs/mapping/gap-webgpu-wit-androidx.md`](docs/mapping/gap-webgpu-wit-androidx.md). P1 W1–W8 smokes landed. Official-shape gap: [`docs/mapping/gap-wasi-p3-wit.md`](docs/mapping/gap-wasi-p3-wit.md).

- Hub freeze, narrow tests, and file whitelist are in the P1 playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.
- **Never file upstream GitHub issues** (or Discussions used as an issue tracker) on WASI, Wasmtime, wasi-webgpu, or any other upstream. No `gh issue create`. Record Android facts only in this repo.

## Cursor Cloud specific instructions

Cloud Agent base images keep rustup **default 1.83.0** even when **1.97.1** is already installed. Wasmtime 47 crates need edition 2024, so `cargo fetch` from `/workspace` with 1.83.0 exits 101 (`feature edition2024 is required`).

Before any `cargo fetch` / `check` / `test` from the repo root: `rustup default 1.97.1`, or `cd native` (see `native/rust-toolchain.toml`). A root `rust-toolchain.toml` pins the same channel when cwd is `/workspace`. Environment `install` must set the default **before** `cargo fetch --locked --manifest-path native/Cargo.toml`.
