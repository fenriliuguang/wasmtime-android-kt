# Agent notes

**Living auto queue:** native Dawn host (full `wasi:webgpu` pin via Dawn C) — [`docs/agent/native-dawn.md`](docs/agent/native-dawn.md) (Cursor skill `native-dawn`). Long branch **`cursor/native-dawn-rewrite-1355`**: one lane = one **commit**; **no PR** until `python3 ./scripts/native-dawn-remaining.py` is empty. Cube / out-of-tree demo is evidence only (`ND-DEVICE`), not consume DoD.

**`0.1.0` product gates** are **empty** ([`docs/agent/product-010.md`](docs/agent/product-010.md); skill `product-010` only if the user names `P010-*`).

P2 Wasmtime pin is **named-only** ([`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md); skill `wasmtime-p2`). Do not run it for 下一刀.

P0 `wasi:webgpu` **shape** is **closed**. Do not re-cut guest-pipeline P1–P5, leftover F1–F9, Dawn consume G1–G9, or WG-6 **as those queues**. Default consume rewrite is **native-dawn** (same pin, Dawn C). Close-out: [`docs/archive/p0-wasi-webgpu.md`](docs/archive/p0-wasi-webgpu.md). WIT ↔ androidx (JNI leftover): [`docs/mapping/gap-webgpu-wit-androidx.md`](docs/mapping/gap-webgpu-wit-androidx.md).

P1 WASI 0.3 official-shape is **closed**. Do not re-cut W1–W8, P1-FS1–FS4, P1-SK1–SK2, P1-HT1, or G-dev. Close-out: [`docs/archive/p1-wasi-p3.md`](docs/archive/p1-wasi-p3.md). `0.1.0` backlog vs named-only: [`docs/mapping/gap-wasi-p3-wit.md`](docs/mapping/gap-wasi-p3-wit.md).

- Hub freeze, narrow tests, and file whitelist are in the **native-dawn** playbook (feature lanes) or the empty 0.1.0 playbook (named `P010-*` only) — do not rediscover them from RFCs or by reading `cm.rs` whole.
- **Never file upstream GitHub issues** (or Discussions used as an issue tracker) on WASI, Wasmtime, wasi-webgpu, or any other upstream. No `gh issue create`. Record Android facts only in this repo.

## Cursor Cloud specific instructions

Cloud Agent base images keep rustup **default 1.83.0** even when **1.97.1** is already installed. Wasmtime 47 crates need edition 2024, so `cargo fetch` from `/workspace` with 1.83.0 exits 101 (`feature edition2024 is required`).

Before any `cargo fetch` / `check` / `test` from the repo root: `rustup default 1.97.1`, or `cd native` (see `native/rust-toolchain.toml`). A root `rust-toolchain.toml` pins the same channel when cwd is `/workspace`. Environment `install` must set the default **before** `cargo fetch --locked --manifest-path native/Cargo.toml`.
