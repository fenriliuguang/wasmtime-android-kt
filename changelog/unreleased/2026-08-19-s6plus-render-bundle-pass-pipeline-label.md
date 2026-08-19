### Code — S6+ render bundle/pass/pipeline label WIT (2026-08-19)

- Hang product `[method]` names: `gpu-render-bundle` / `gpu-render-bundle-encoder` / `gpu-render-pass-encoder` label + set-label and `gpu-render-pipeline` label + set-label + get-bind-group-layout
- Guest lifts WIT string getters/setters or own bind-group-layout (drop); export `run` returns harness `1`; L2 unused (empty labels; no new JNI)
- Fixtures `webgpu_method_render_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
