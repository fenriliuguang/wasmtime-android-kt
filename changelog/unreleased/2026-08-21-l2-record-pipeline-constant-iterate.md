### Code — L2 record-gpu-pipeline-constant-value iterate guest fields to host (2026-08-21)

- Deepen `[method]record-gpu-pipeline-constant-value.keys` / `values` / `entries` described JNI: guest resource rep via count + indexed get (same family as compilation-info messages); maps stay in `AbiCmHostBindings`
- Guest fixtures still call iterate on a fresh empty record; host returns empty lists; export `run` returns harness `1`
- Fixtures `webgpu_method_record_gpu_pipeline_constant_value_{keys,values,entries}`; native modules of the same names
