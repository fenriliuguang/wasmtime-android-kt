### Code — L2 record-option-gpu-size64 iterate guest fields to host (2026-08-21)

- Deepen `[method]record-option-gpu-size64.keys` / `values` / `entries` described JNI: guest resource rep via count + indexed get; `option<u64>` uses state `0` none / `1` some; maps stay in `AbiCmHostBindings`
- Guest fixtures still call iterate on a fresh empty record; host returns empty lists; export `run` returns harness `1`
- Fixtures `webgpu_method_record_option_gpu_size64_{keys,values,entries}`; native modules of the same names
