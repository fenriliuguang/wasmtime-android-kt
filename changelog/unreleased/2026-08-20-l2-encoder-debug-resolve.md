### Code — L2 gpu-command-encoder debug and resolve-query-set guest fields to host (2026-08-20)

- Deepen `[method]gpu-command-encoder.resolve-query-set` / `push-debug-group` / `pop-debug-group` / `insert-debug-marker` from lift-only stubs to described JNI (encoder handle + labels/indices → Dawn/Cpu)
- Guest `get-encoder` still uses rep 0; the wrap stub-creates an encoder; query-set / destination 0 → stub in the attach; export `run` returns harness `1`
- New host APIs: `commandEncoderResolveQuerySet` / `PushDebugGroup` / `PopDebugGroup` / `InsertDebugMarker`
