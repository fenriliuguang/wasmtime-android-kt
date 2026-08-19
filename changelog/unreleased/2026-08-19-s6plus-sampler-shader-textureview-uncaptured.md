### Code — S6+ sampler/shader/texture-view/uncaptured methods (2026-08-19)

- Hang product `[method]` names: `gpu-sampler.label` / `set-label` / `gpu-shader-module.get-compilation-info` / `label` / `set-label` / `gpu-texture-view.label` / `set-label` / `gpu-uncaptured-error-event.error`
- Guest lifts WIT types via test constructors; export `run` returns harness `1`; L2 unused (lift-only stubs; `get-compilation-info` true CM async own; no new JNI)
- Fixtures `webgpu_method_sampler_*` / `shader_module_*` / `texture_view_*` / `uncaptured_error_event_error`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
