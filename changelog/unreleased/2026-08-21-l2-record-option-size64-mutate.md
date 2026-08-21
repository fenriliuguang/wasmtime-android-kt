### Code — L2 record-option-gpu-size64 mutate guest fields to host (2026-08-21)

- Deepen `[method]record-option-gpu-size64.add` / `get` / `has` / `remove` described JNI: guest resource rep + key + `option<u64>` (`hasValue` + `Long`); host stores maps in `AbiCmHostBindings`; iterate stays lift-only
- Guest fixtures still pass empty key / value none; `get`/`has` on a fresh record yield none/false; export `run` returns harness `1`
- Fixtures `webgpu_method_record_option_gpu_size64_{add,get,has,remove}`; native modules of the same names
