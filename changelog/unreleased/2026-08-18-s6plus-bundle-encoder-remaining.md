### Code — S6+ remaining render-bundle-encoder recording WIT (2026-08-18)

- Hang remaining render-bundle-encoder recording commands that still lacked product `[method]` names: `set-index-buffer` / `set-vertex-buffer` / `draw-indexed` / `draw-indirect` / `draw-indexed-indirect` / `push-debug-group` / `pop-debug-group` / `insert-debug-marker` / `set-immediates`
- Guest lifts WIT buffer borrows, index format, draw options, empty debug strings, and empty immediates; export `run` returns harness `1`; L2 unused (lift-only, no new JNI)
- Fixtures `webgpu_method_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
