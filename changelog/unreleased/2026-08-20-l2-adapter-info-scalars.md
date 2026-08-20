### Code — L2 gpu-adapter-info scalar getters guest fields to host (2026-08-20)

- Deepen `[method]gpu-adapter-info.subgroup-min-size` / `subgroup-max-size` / `is-fallback-adapter` from omit-lane lift-only stubs to described JNI with guest adapter handle
- `GpuAdapterInfo` stores the owning adapter rep; `get-adapter-info` still pushes `adapter: 0` and getters stub-request an adapter when needed
- New host APIs `adapterInfoSubgroupMinSize` / `SubgroupMaxSize` / `IsFallbackAdapter` and `deviceAdapter` for `gpu-device.adapter-info`
