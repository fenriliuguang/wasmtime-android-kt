### Code — L2 create-*-pipeline constants guest record to host (2026-08-21)

- Pass `record-gpu-pipeline-constant-value` **rep** into `[method]gpu-device.create-compute-pipeline` / `create-render-pipeline` (and async twins); 0 stays none
- Do not re-cut the map resource; host snapshots the existing keyed f64 map onto `ProgrammableStage` / vertex / fragment
- Fixture `webgpu_method_create_compute_pipeline_constants` (add `c`=1.0); native `create_compute_pipeline_constants`
