### Code — S4 [method]gpu-device.create-buffer takes gpu-buffer-descriptor (2026-08-17)

- Replace transitional u32 getter with WIT-shaped `(borrow<gpu-device>, gpu-buffer-descriptor) -> own<gpu-buffer>`
- Guest passes size=4, usage=COPY_DST|VERTEX (mapped/label=none); host table stores L2 handle in `GpuBuffer.rep`; drops the own handle; export `run` returns harness `1`
- Fixture `fixtures/w1/webgpu_method_create_buffer`; native `wasi_webgpu_method_create_buffer`; twin instrument `WasiWebGpuMethodCreateBufferInstrumentedTest`
