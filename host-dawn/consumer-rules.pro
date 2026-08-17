# Keep SPI + documented Dawn entry so R8 cannot strip ServiceLoader.
-keep class io.github.fenriliuguang.wasmtime.android.host.dawn.DawnWebGpuBackendFactory { *; }
-keep class io.github.fenriliuguang.wasmtime.android.host.dawn.GpuBackends { *; }
-keep class io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendFactory { *; }
