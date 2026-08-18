### Code — S6+ pipeline-async / mapped-range WIT (2026-08-18)

- Hang `create-render-pipeline-async` / `create-compute-pipeline-async` (`async result<own<pipeline>, create-pipeline-error>`) and `get-mapped-range-get-with-copy` / `get-mapped-range-set-with-copy` (`result<list<u8>, …>` / `result<_, …>`)
- Guest reuses the pipeline descriptor graph (shader borrow + `layout=auto`) or passes mapped-range offset/size none + empty set data; drops owns on ok; export `run` returns harness `1`; L2 stays host-fixed (pipeline JNI reused; mapped-range returns empty list / ok without new JNI)
- Fixtures `webgpu_method_create_render_pipeline_async` / `webgpu_method_create_compute_pipeline_async` / `webgpu_method_buffer_get_mapped_range` / `webgpu_method_buffer_set_mapped_range`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
