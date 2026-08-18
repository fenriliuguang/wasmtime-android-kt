### Code — S6+ pipeline create descriptors / pass-end harness (2026-08-18)

- Cut the remaining `create-*-pipeline` methods that still returned transitional `u32`, plus the two already-WIT `*-pass-end` guests that still returned stub reps: `create-render-pipeline` / `create-compute-pipeline` now take WIT descriptors and return `own<pipeline>`; `render-pass-end` / `compute-pass-end` keep void and return harness `1`
- Guest passes shader borrow + `layout=auto` (render also leaves buffers/primitive/depth/ms/fragment none); drops owns; export `run` returns harness `1`; L2 stays host-fixed stub shader + triangle / empty layout
- Fixtures `webgpu_method_create_render_pipeline` / `webgpu_method_create_compute_pipeline` / `webgpu_method_render_pass_end` / `webgpu_method_compute_pass_end`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
