package io.github.fenriliuguang.wasmtime.android.host.dawn

import io.github.fenriliuguang.wasi.webgpu.experimental.dawn.DawnWasiWebGpuHost
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CpuWasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackend
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendFactory

/**
 * Documented factory so R8 cannot strip Dawn when apps use
 * `store.setWebGpuBackend(GpuBackends.dawn())` instead of ServiceLoader
 * (`Store.createWithDiscoveredBackend`). Explicit attach always wins.
 */
object GpuBackends {
    const val DAWN_ID: String = "dawn"
    const val CPU_ID: String = "cpu"

    fun dawn(): WebGpuBackend =
        HostWebGpuBackend(DawnWasiWebGpuHost.create(), id = DAWN_ID)

    fun cpu(): WebGpuBackend =
        HostWebGpuBackend(CpuWasiWebGpuHost(), id = CPU_ID)
}

class DawnWebGpuBackendFactory : WebGpuBackendFactory {
    override fun create(): WebGpuBackend = GpuBackends.dawn()
}
