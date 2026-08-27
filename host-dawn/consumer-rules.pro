# Published R8 contract (L5 / P010-DISC): keep SPI + documented Dawn entry
# so R8 cannot strip ServiceLoader (`Store.createWithDiscoveredBackend`).
-keep class io.github.fenriliuguang.wasmtime.android.host.dawn.DawnWebGpuBackendFactory { *; }
-keep class io.github.fenriliuguang.wasmtime.android.host.dawn.GpuBackends { *; }
-keep class io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendFactory { *; }
