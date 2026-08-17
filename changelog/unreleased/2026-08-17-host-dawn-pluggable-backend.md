### Code — pluggable GPU backend modules + unwired request-adapter none (2026-08-17)

- Split `:host-dawn` (Dawn adapter + unpublished mavenLocal host) and `:android-webgpu` (default bundle). `:runtime-jni` no longer `api()`-depends on `host-api` / `abi-cm`
- SPI: `WebGpuBackend` / `WebGpuBackendFactory` in `runtime-api`; `Store.setWebGpuBackend` / optional ServiceLoader; `GpuBackends.dawn()`
- Unwired `[method]gpu.request-adapter` returns guest `none` (flat `request-adapter` returns `0`); no trap `experimental host callback not set`
- `smoke-app` depends on `:android-webgpu`. **Vendor path decided:** copy mvp Host Kotlin into this repo; Dawn `.so` via `androidx.webgpu`; mavenLocal `experimental:*` is transitional ([`docs/blocked-gpu-host.md`](../../docs/blocked-gpu-host.md))
