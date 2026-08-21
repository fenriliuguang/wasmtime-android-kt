### Code — leftover descriptor semantics request-adapter xr-compatible (2026-08-21)

- JNI `requestAdapterDescribed` now packs guest `xr-compatible` (`-1` none / `0` false / `1` true) with existing power-preference / force-fallback / feature-level
- Fixture `webgpu_method_request_adapter` passes `xr-compatible=true`; native smoke asserts the lifted option (true async wrap kept)
- Dawn `GPURequestAdapterOptions` stays power/fallback/backend this cut; `xrCompatible` stays on the Kotlin record if androidx omits the ctor slot
