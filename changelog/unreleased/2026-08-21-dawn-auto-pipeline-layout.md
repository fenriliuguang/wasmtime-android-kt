### Code — L2 auto pipeline layout (2026-08-21)

- `create-compute-pipeline` with WIT `layout: auto` no longer throws `auto pipeline layout; pass an explicit pipeline-layout handle`; Dawn omits `GPUPipelineLayout` (androidx 2-arg `GPUComputePipelineDescriptor`)
- Fixture `webgpu_method_create_compute_pipeline` already lifts `layout=auto`; explicit pipeline-layout handles stay valid; render still uses an explicit empty layout (it did not throw this sentinel)
- androidx `1.0.0-alpha05` exposes auto by omitting the layout ctor argument (`null`)
