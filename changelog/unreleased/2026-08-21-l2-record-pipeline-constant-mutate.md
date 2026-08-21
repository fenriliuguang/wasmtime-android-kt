### Code — L2 record-gpu-pipeline-constant-value mutate guest fields to host (2026-08-21)

- Deepen `[method]record-gpu-pipeline-constant-value.add` / `get` / `has` / `remove` described JNI: guest resource rep + key (`HostArg::Str`) + f64; host stores maps in `AbiCmHostBindings`; iterate (`keys`/`values`/`entries`) stays lift-only
- Guest fixtures still pass empty key / `0.0`; `get`/`has` on a fresh record yield none/false; export `run` returns harness `1`
- Fixtures `webgpu_method_record_gpu_pipeline_constant_value_{add,get,has,remove}`; native modules of the same names
