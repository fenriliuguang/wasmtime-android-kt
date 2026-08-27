# Keep JNI entry points if consumers minify.
-keep class io.github.fenriliuguang.wasmtime.android.jni.NativeBridge { *; }
# Published R8 contract (L5 / P010-DISC): ServiceLoader + documented Dawn entry.
-keep class io.github.fenriliuguang.wasmtime.android.host.dawn.DawnWebGpuBackendFactory { *; }
-keep class io.github.fenriliuguang.wasmtime.android.host.dawn.GpuBackends { *; }
-keep class io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendFactory { *; }
