# Dawn guest render (WG-6)

- Guest `webgpu_method_dawn_guest_render` 走 VERTEX buffer + `draw`(3) + `queue.submit`，打到 `DawnWasiWebGpuHost`；不是 1×1 color-clear cite，也不是 `@builtin(vertex_index)` 离屏三角。
- `ExperimentalWebGpuBridge.attachDawnGuestRender` 组合转发 described 回调（真 pipeline + set-vertex-buffer，不 stub triangle）；instrument `WasiWebGpuDawnGuestRenderInstrumentedTest` 资产跑夹具。
- Native `dawn_guest_render`：shader `@location(0)`、buffer size=36 VERTEX、pipeline float32x3 + layout auto + depth-stencil none、set-vb slot 0、draw 3、submit。
