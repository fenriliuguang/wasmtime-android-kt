### Code — vendor Dawn host Kotlin into :host-dawn (2026-08-17)

- Copy mvp `host-api` / `DawnWasiWebGpuHost` / `AbiCmHostBindings` into `:host-dawn` (packages stay `…experimental…`)
- `:host-dawn` depends on published `androidx.webgpu:webgpu:1.0.0-alpha05` for Dawn `.so`; drop mavenLocal `experimental:*` and `settings.gradle.kts` `mavenLocal()`
- Licenses: [`docs/blocked-gpu-host.md`](../../docs/blocked-gpu-host.md), [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md), [`host-dawn/third_party/wasi-webgpu-jvm-mvp/`](../../host-dawn/third_party/wasi-webgpu-jvm-mvp/)
