### Code — S8 option-none → own cluster (2026-08-17)

- Cut three remaining WIT `option<descriptor> → own<resource>` methods in one slice: `create-sampler`, `texture.create-view`, `begin-compute-pass`
- Guest passes descriptor=none; drops the own handle; export `run` returns harness `1`; L2 stays host-fixed
- Fixtures `webgpu_method_create_sampler` / `webgpu_method_texture_create_view` / `webgpu_method_begin_compute_pass`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
