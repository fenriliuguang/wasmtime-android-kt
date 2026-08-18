### Code — S6+ remaining compute-pass recording and pass debug WIT (2026-08-18)

- Hang remaining compute-pass recording plus pass debug that still lacked product `[method]` names: `dispatch-workgroups-indirect` / `set-immediates` / compute-pass `push-debug-group` / `pop-debug-group` / `insert-debug-marker` and render-pass `push-debug-group` / `pop-debug-group` / `insert-debug-marker`
- Guest lifts WIT buffer borrow, empty immediates list, and debug strings; export `run` returns harness `1`; L2 stays host-fixed (`dispatch-workgroups-indirect` reuses dispatch JNI; set-immediates/debug are lift-only, no new JNI)
- Fixtures `webgpu_method_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
