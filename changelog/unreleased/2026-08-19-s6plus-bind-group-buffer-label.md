### Code — S6+ bind-group / layout / buffer label WIT (2026-08-19)

- Hang product `[method]` names: `gpu-bind-group.label` / `set-label`, `gpu-bind-group-layout.label` / `set-label`, and `gpu-buffer.label` / `set-label` / `size` / `usage` / `map-state`
- Guest lifts WIT string getters/setters, `u64` size, `gpu-buffer-usage` flags, and `gpu-buffer-map-state`; export `run` returns harness `1`; L2 unused (empty labels, size 0, empty usage, unmapped; no new JNI)
- Fixtures `webgpu_method_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
