### Code — S6+ wgsl-language-features.has (2026-08-19)

- Hang product `[method]` name: `wgsl-language-features.has`
- Guest lifts WIT types via `get-gpu` + `gpu.wgsl-language-features`; export `run` returns harness `1`; L2 unused (lift-only stub returns false; no new JNI)
- Fixture `webgpu_method_wgsl_language_features_has`; native module under `wasi_webgpu_method/`; twin instrument asserts harness `1`
