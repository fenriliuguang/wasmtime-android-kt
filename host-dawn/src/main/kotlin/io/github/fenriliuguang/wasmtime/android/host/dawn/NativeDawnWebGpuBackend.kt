package io.github.fenriliuguang.wasmtime.android.host.dawn

import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackend

/**
 * Product `id = "dawn"` backend. Selects in-process NativeGpu via
 * `Store.setWebGpuBackend` — not a Kotlin WebGPU client (NG-3).
 */
class NativeDawnWebGpuBackend : WebGpuBackend {
    override val id: String = GpuBackends.DAWN_ID

    override fun close() {}
}
