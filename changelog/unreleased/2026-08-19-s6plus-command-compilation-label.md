### Code — S6+ command buffer/encoder label and compilation WIT (2026-08-19)

- Hang product `[method]` names: `gpu-command-buffer.label` / `set-label`, `gpu-command-encoder.label` / `set-label`, `gpu-compilation-info.messages`, and `gpu-compilation-message.message` / `type` / `line-num` / `line-pos` / `offset` / `length`
- Guest lifts WIT strings, empty `list<own<gpu-compilation-message>>`, enum, and `u64` getters; export `run` returns harness `1`; L2 unused (empty labels/list/message, type error, numeric fields 0; no new JNI)
- Fixtures `webgpu_method_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
