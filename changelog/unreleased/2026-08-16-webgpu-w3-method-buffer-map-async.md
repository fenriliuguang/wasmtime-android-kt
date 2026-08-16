### Code — wasi:webgpu W3 [method]gpu-buffer.map-async (2026-08-16)

- Register `gpu-buffer` / `get-buffer` + `[method]gpu-buffer.map-async` (true CM async void via `func_wrap_concurrent` + oneshot yield; L2 adapter → device → host-fixed MAP_READ buffer then map; Guest stub buffer ignored)
- Fixture `fixtures/w1/webgpu_method_buffer_map_async`; native `wasi_webgpu_method_buffer_map_async`; twin instrument `WasiWebGpuMethodBufferMapAsyncInstrumentedTest`
- Transitional: Guest returns stub 31. Not proposal `result` / mode / offset. Not compliance
