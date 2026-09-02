### Gfx — surface size and resize (2026-09-02)

- `wasi-gfx:surface` `height` / `width` follow the bound window (`bindCanvasNativeWindow`); `request-set-size` updates that window record (NativeGpu swapchain size) and `on-resize` yields the latest `{height, width}`. Cloud / no window uses constructor desc then the request.
