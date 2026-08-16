### Code — S1 [method]gpu-device.queue returns own<gpu-queue> (2026-08-16)

- Replace transitional u32 getter with WIT-shaped `(borrow<gpu-device>) -> own<gpu-queue>`; host table stores L2 handle in `GpuQueue.rep`
- Guest drops the own handle; export `run` returns harness `1` (not the method shape)
- Fixture `fixtures/w1/webgpu_method_device_queue`; native `wasi_webgpu_method_device_queue`; twin instrument `WasiWebGpuMethodDeviceQueueInstrumentedTest`
