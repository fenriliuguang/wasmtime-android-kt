### Code — S5 [method]gpu-queue.submit takes list<borrow<gpu-command-buffer>> (2026-08-17)

- Replace transitional single-u32 submit with WIT-shaped `(borrow<gpu-queue>, list<borrow<gpu-command-buffer>>) -> ()`
- Guest passes a one-element list from `get-command-buffer`; host `table.get`s each list element; L2 still host-fixed encoder → finish → `submit1`; drops owns; export `run` returns harness `1`
- Fixture `fixtures/w1/webgpu_method_queue_submit`; native `wasi_webgpu_method_queue_submit`; twin instrument `WasiWebGpuMethodQueueSubmitInstrumentedTest`
