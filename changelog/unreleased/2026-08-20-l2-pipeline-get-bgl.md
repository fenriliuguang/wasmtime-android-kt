### Code — L2 pipeline get-bind-group-layout guest fields to host (2026-08-20)

- Deepen `[method]gpu-render-pipeline.get-bind-group-layout` / `[method]gpu-compute-pipeline.get-bind-group-layout` from lift-only stubs to described JNI (pipeline rep + group index → host BGL rep stored on the guest resource)
- Guest constructors still use rep 0; the attaches stub-create a triangle / compute pipeline when needed
- New host APIs `renderPipelineGetBindGroupLayout` / `computePipelineGetBindGroupLayout` (Dawn `getBindGroupLayout`)
