### Code — wasi:webgpu W3 [method]gpu-queue.submit (2026-08-15)

- Register WIT `gpu-queue` + `get-queue` + `[method]gpu-queue.submit` (sync void; single command-buffer u32, not proposal `list`; L2 adapter → device → queue + encoder → finish → submit1)
- Fixture `fixtures/w1/webgpu_method_queue_submit`; native `wasi_webgpu_method_queue_submit`; twin instrument `WasiWebGpuMethodQueueSubmitInstrumentedTest`
- Flat `queue-submit1` kept. Completes the W3 high-frequency `[method]` surface from the gap table. Not compliance
