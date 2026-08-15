### Code — wasi:webgpu W3 [method]gpu-device.queue (2026-08-15)

- Register WIT `gpu-device` resource + `get-device` + `[method]gpu-device.queue` (sync getter; L2 `request-adapter` then `adapter-request-device` then `device-get-queue`, same u32 as flat `device-get-queue`)
- Fixture `fixtures/w1/webgpu_method_device_queue`; native `wasi_webgpu_method_device_queue`; twin instrument `WasiWebGpuMethodDeviceQueueInstrumentedTest`
- Transitional: method still returns u32 (not `gpu-queue` resource). Flat `device-get-queue` kept. Not compliance
