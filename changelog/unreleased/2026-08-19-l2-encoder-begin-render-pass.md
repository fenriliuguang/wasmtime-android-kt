### Code — L2 gpu-command-encoder begin-render-pass guest fields to host (2026-08-19)

- Deepen `[method]gpu-command-encoder.begin-render-pass` from host-fixed stub view `23` to described JNI (first color-attachment view + load-op + store-op)
- Guest begins a pass with one color-attachment (`load-op=clear`, `store-op=store`); native wrap uses encoder/view `rep` when non-zero and forwards Dawn load/store into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_begin_render_pass`; native module `begin_render_pass`
