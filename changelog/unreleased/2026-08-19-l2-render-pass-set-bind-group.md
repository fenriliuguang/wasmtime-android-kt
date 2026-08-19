### Code — L2 gpu-render-pass-encoder set-bind-group guest fields to host (2026-08-19)

- Deepen `[method]gpu-render-pass-encoder.set-bind-group` from host-fixed rebuild+empty group to described JNI (pass/bind-group reps + index; offsets none → empty)
- Guest passes index `0`, bind-group some, offsets/start/length none; native wrap uses pass `rep` when non-zero and forwards Dawn ints into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_render_pass_set_bind_group`; native module `render_pass_set_bind_group`
