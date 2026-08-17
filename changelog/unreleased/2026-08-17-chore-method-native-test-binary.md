### Chore — fold [method] native tests into one binary (2026-08-17)

- Move implemented `native/tests/wasi_webgpu_method_*.rs` into `native/tests/wasi_webgpu_method/` so `cargo test --tests` links one harness instead of 32
- Flat W1 smokes stay as separate binaries; instrument twins unchanged
