### Code — S6+ gpu-canvas-context take WIT types (2026-08-20)

- Hang product `[method]gpu-canvas-context.configure` / `unconfigure` / `get-configuration` / `get-current-texture` plus test-only `get-canvas-context`
- Guest lifts WIT canvas records; export `run` returns harness `1`; L2 unused (lift-only stubs; no new JNI / no product `surface-*`)
- Fixtures `webgpu_method_canvas_context_*`; native modules of the same names; twin instruments assert harness `1`
