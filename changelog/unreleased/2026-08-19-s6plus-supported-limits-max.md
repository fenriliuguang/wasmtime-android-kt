### Code — S6+ gpu-supported-limits max getters WIT (2026-08-19)

- Hang product `[method]` names: `gpu-supported-limits.max-bind-groups` / `max-bind-groups-plus-vertex-buffers` / `max-bindings-per-bind-group` / `max-buffer-size` / `max-color-attachment-bytes-per-sample` / `max-color-attachments` / `max-compute-invocations-per-workgroup` / `max-compute-workgroup-size-x` / `max-compute-workgroup-size-y` / `max-compute-workgroup-size-z` / `max-compute-workgroups-per-dimension` / `max-compute-workgroup-storage-size`
- Guest lifts WIT numerics via `get-supported-limits`; export `run` returns harness `1`; L2 unused (lift-only stub `1` / `1u64`; no new JNI)
- Fixtures `webgpu_method_supported_limits_max_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
