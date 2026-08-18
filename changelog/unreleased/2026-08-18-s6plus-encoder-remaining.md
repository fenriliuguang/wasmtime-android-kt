### Code — S6+ remaining command-encoder recording WIT (2026-08-18)

- Hang the remaining command-encoder recording commands that still lacked product `[method]` names: `copy-buffer-to-texture` / `copy-texture-to-buffer` / `copy-texture-to-texture` / `clear-buffer` / `resolve-query-set` / `push-debug-group` / `pop-debug-group` / `insert-debug-marker`
- Guest lifts WIT texel-copy records, buffer/query-set borrows, and debug strings; export `run` returns harness `1`; L2 stays host-fixed (copy/clear reuse buffer-copy JNI; resolve/debug are lift-only, no new JNI)
- Fixtures `webgpu_method_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
