package io.github.fenriliuguang.wasmtime.android.host.dawn

import io.github.fenriliuguang.wasi.webgpu.experimental.dawn.DawnWasiWebGpuHost
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CpuWasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackend
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendFactory

/**
 * Documented factory so R8 cannot strip Dawn when apps use
 * `store.setWebGpuBackend(GpuBackends.dawn())` instead of ServiceLoader
 * (`Store.createWithDiscoveredBackend`). Explicit attach always wins.
 *
 * [dawn] is NativeGpu (Dawn C). androidx JNI leftover is [dawnJni]
 * (`id = "dawn-jni"`).
 */
object GpuBackends {
    const val DAWN_ID: String = "dawn"
    const val DAWN_JNI_ID: String = "dawn-jni"
    const val CPU_ID: String = "cpu"

    /** Product default: in-process NativeGpu. Does not attach the JNI table. */
    fun dawn(): WebGpuBackend = NativeDawnWebGpuBackend()

    /** androidx / `DawnWasiWebGpuHost` leftover. Explicit BYO. */
    fun dawnJni(): WebGpuBackend =
        HostWebGpuBackend(DawnWasiWebGpuHost.create(), id = DAWN_JNI_ID)

    fun cpu(): WebGpuBackend =
        HostWebGpuBackend(CpuWasiWebGpuHost(), id = CPU_ID)
}

class DawnWebGpuBackendFactory : WebGpuBackendFactory {
    override fun create(): WebGpuBackend = GpuBackends.dawn()
}
