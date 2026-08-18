### Code — S6+ remaining render-pass recording WIT (2026-08-18)

- Hang the remaining render-pass recording commands that still lacked product `[method]` names: `set-viewport` / `set-scissor-rect` / `set-blend-constant` / `set-stencil-reference` / `set-index-buffer` / `draw-indexed` / `draw-indirect` / `draw-indexed-indirect`
- Guest lifts WIT scalars, `gpu-color`, `gpu-index-format`, and buffer borrows; export `run` returns harness `1`; L2 stays host-fixed (draw / vertex-buffer JNI reused; state setters lift-only, no new JNI)
- Fixtures `webgpu_method_render_pass_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
