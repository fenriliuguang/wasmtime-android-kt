### Code — S6+ pass recording / copy-buffer cluster (2026-08-18)

- Cut eight remaining pass / encoder recording methods that still took transitional `u32`: `set-pipeline` (compute + render), `set-bind-group` (compute + render), `dispatch-workgroups`, `set-vertex-buffer`, `draw`, `copy-buffer-to-buffer`
- Guest passes WIT borrows / options (`set-bind-group` → `result<_, set-bind-group-error>`); drops extra owns; export `run` returns harness `1`; L2 stays host-fixed pipeline / empty bind-group / VERTEX slot 0 / draw(3) / 1×1×1 / 4-byte copy
- Fixtures `webgpu_method_compute_pass_set_pipeline` / `webgpu_method_compute_pass_set_bind_group` / `webgpu_method_compute_pass_dispatch_workgroups` / `webgpu_method_render_pass_set_pipeline` / `webgpu_method_render_pass_set_bind_group` / `webgpu_method_render_pass_set_vertex_buffer` / `webgpu_method_render_pass_draw` / `webgpu_method_copy_buffer_to_buffer`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
