# Agent notes

**Living leftover:** Dawn C full bind → wasi-gfx size/resize → remaining pin input streams — [`docs/agent/remaining.md`](docs/agent/remaining.md). Run `python3 ./scripts/remaining.py` (one printed **Next:** only).

Product coordinate is **`0.1.0`** (not pressed). Do not bump the GAV for a consume-path change.

P2 Wasmtime pin is **named-only** ([`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md)). Do not run it for 下一刀.

- Guest WIT names stay. Reuse `cm.rs` lowering. Grep then Read ~80 lines. Do not reimplement `exp_*` JNI.
- Named-only (never auto): `context.unconfigure`, timestamped `frame-event`, Lost/Outdated `result`, multi-window, G-cmd, G-fs-full, listen/UDP, wasi-testsuite, `wasmtime-wasi`, CTS, this-repo 1.0.
- **Never file upstream GitHub issues** (or Discussions used as an issue tracker) on WASI, Wasmtime, wasi-webgpu, or any other upstream. No `gh issue create`. Record Android facts only in this repo.

## Cursor Cloud specific instructions

Cloud Agent base images keep rustup **default 1.83.0** even when **1.97.1** is already installed. Wasmtime 47 crates need edition 2024, so `cargo fetch` from `/workspace` with 1.83.0 exits 101 (`feature edition2024 is required`).

Before any `cargo fetch` / `check` / `test` from the repo root: `rustup default 1.97.1`, or `cd native` (see `native/rust-toolchain.toml`). A root `rust-toolchain.toml` pins the same channel when cwd is `/workspace`. Environment `install` must set the default **before** `cargo fetch --locked --manifest-path native/Cargo.toml`.
