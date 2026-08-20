### Code — L2 gpu-compute-pass-encoder debug and set-immediates guest fields to host (2026-08-20)

- Deepen `[method]gpu-compute-pass-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` / `set-immediates` from lift-only stubs to described JNI (pass handle + labels / immediates bytes → Dawn/Cpu)
- Guest `get-compute-pass` still uses rep 0; the wraps stub-build adapter→device→encoder→compute-pass when needed
- New host APIs `computePassPushDebugGroup` / `PopDebugGroup` / `InsertDebugMarker` / `SetImmediates` (Dawn validates the handle for set-immediates; alpha05 lacks the entry point)
