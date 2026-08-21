### Code — L2 dawn consume default-queue (2026-08-21)

- Dawn `GPUDeviceDescriptor.defaultQueue` now takes `GPUQueueDescriptor(label=…)` from snapshotted `defaultQueueLabel` (absent → none)
- Do not re-cut F7 JNI; fixture `webgpu_method_request_device` already lifts `default-queue.label=l2`; keep required-features / required-limits / callbacks
- androidx `1.0.0-alpha05` exposes the `defaultQueue` ctor slot
