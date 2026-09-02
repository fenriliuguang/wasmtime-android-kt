### Fix — do not `create_texture` then `set_dawn` on swapchain acquire (2026-09-02)

- Regression after `fb57f2b` (`#303` Dawn C full bind). Through `f9f24c1`, `NativeGpu::create_texture` only inserted a table row with Dawn slot `0`, so `canvas_current_texture` could `set_dawn` the BLAST image without allocating. Full bind made `create_texture` call `wgpuDeviceCreateTexture`; each `get-current-texture` then leaked a full-size GPU texture (smooth → hitch → device freeze on the out-of-tree cube).
- Acquire now inserts the surface texture directly. keep-3 retire `try_drop`s that image plus its views and command buffers (same as `DawnWasiWebGpuHost`).
- Device: Vivo V2458A / arm64 / Android 16, out-of-tree rotating-cube >3 min, Choreographer vsync ~8.33 ms.
