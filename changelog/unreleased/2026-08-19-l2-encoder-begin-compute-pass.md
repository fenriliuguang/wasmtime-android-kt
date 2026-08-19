### Code — L2 gpu-command-encoder begin-compute-pass guest fields to host (2026-08-19)

- Deepen `[method]gpu-command-encoder.begin-compute-pass` from host-fixed default descriptor to described JNI (timestamp-write indices)
- Guest begins a compute pass with `timestamp-writes` beginning=0 / end=1; native wrap uses encoder `rep` when non-zero and forwards the indices into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_begin_compute_pass`; native module `begin_compute_pass`
