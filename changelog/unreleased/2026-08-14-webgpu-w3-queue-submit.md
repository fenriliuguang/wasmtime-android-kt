### Code — wasi:webgpu W3 queue-submit1 dual-register (2026-08-14)

- Dual-register transitional flat `wasi:webgpu/webgpu@0.3.0-rc.2#queue-submit1` (sync void; same L2 as experimental; single command-buffer u32, not proposal `list`)
- Fixture `fixtures/w1/webgpu_queue_submit`; native `wasi_webgpu_queue_submit`; twin instrument `WasiWebGpuQueueSubmitInstrumentedTest`
- Not `[method]gpu-queue.submit`; not begin-render-pass; not compliance
