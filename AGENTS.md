# Agent notes

**P2 Wasmtime pin** (knowable, upgradeable, rollback-able): follow [`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md) (Cursor skill `wasmtime-p2`). Next PR: `.\scripts\wasmtime-p2-remaining.ps1` or `python ./scripts/wasmtime-p2-remaining.py`.

**L5 productization is accepted** ([`docs/scheme/rfc-l5-productization.md`](docs/scheme/rfc-l5-productization.md)): class B, perpetual `0.x`, Central at `0.1.0`. Frame loop: [`docs/scheme/rfc-wasi-gfx-frame-loop.md`](docs/scheme/rfc-wasi-gfx-frame-loop.md). Neither is `wasmtime-p2-remaining` `Next:`.

P0 `wasi:webgpu` is **closed**. Do not re-cut guest-pipeline P1–P5, leftover F1–F9, Dawn consume G1–G9, or WG-6. Close-out: [`docs/archive/p0-wasi-webgpu.md`](docs/archive/p0-wasi-webgpu.md). WIT ↔ androidx holes: [`docs/mapping/gap-webgpu-wit-androidx.md`](docs/mapping/gap-webgpu-wit-androidx.md).

P1 WASI 0.3 official-shape is **closed**. Do not re-cut W1–W8, P1-FS1–FS4, P1-SK1–SK2, P1-HT1, or G-dev. Close-out: [`docs/archive/p1-wasi-p3.md`](docs/archive/p1-wasi-p3.md). Named leftovers: [`docs/mapping/gap-wasi-p3-wit.md`](docs/mapping/gap-wasi-p3-wit.md).

- Hub freeze, narrow tests, and file whitelist are in the P2 playbook — do not rediscover them from RFCs or by reading `cm.rs` whole.
- **Never file upstream GitHub issues** (or Discussions used as an issue tracker) on WASI, Wasmtime, wasi-webgpu, or any other upstream. No `gh issue create`. Record Android facts only in this repo.

## Cursor Cloud specific instructions

Cloud Agent base images keep rustup **default 1.83.0** even when **1.97.1** is already installed. Wasmtime 47 crates need edition 2024, so `cargo fetch` from `/workspace` with 1.83.0 exits 101 (`feature edition2024 is required`).

Before any `cargo fetch` / `check` / `test` from the repo root: `rustup default 1.97.1`, or `cd native` (see `native/rust-toolchain.toml`). A root `rust-toolchain.toml` pins the same channel when cwd is `/workspace`. Environment `install` must set the default **before** `cargo fetch --locked --manifest-path native/Cargo.toml`.
