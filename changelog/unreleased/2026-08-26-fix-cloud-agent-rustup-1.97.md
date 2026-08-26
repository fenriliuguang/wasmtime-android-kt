### Fix — Cloud Agent cargo uses Rust 1.97.1 at repo root (2026-08-26)

- Add root `rust-toolchain.toml` matching `native/rust-toolchain.toml` so rustup from `/workspace` selects 1.97.1 (Wasmtime 47 crates need edition 2024; Cloud Agent default cargo 1.83.0 fails `cargo fetch`)
- Document the rustup-default gotcha in `AGENTS.md` and `docs/build.md`; environment install must `rustup default 1.97.1` before `cargo fetch`
