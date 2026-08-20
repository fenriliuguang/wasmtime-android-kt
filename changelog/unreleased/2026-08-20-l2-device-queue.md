### Code — L2 gpu-device queue guest fields to host (2026-08-20)

- Deepen `[method]gpu-device.queue` so a non-zero guest `device.rep` is passed to described JNI (`deviceGetQueueDescribed`) with **no** `request-adapter` → `request-device` rebuild
- `get-device` still pushes `rep` 0; the wrap stub-creates only on that fixture path; export `run` returns harness `1`
- Fixture `webgpu_method_device_queue`; native module of the same name
