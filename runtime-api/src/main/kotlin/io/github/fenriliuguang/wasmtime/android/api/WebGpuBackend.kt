package io.github.fenriliuguang.wasmtime.android.api

/**
 * Pluggable GPU host owned by this repo. Third parties implement this SPI;
 * they do not re-export `wasi:webgpu` WIT.
 *
 * [hostCallbacks] is the L1 attach surface (flat u32-rep JNI). Missing backend is
 * not a link error: `gpu.request-adapter` returns guest `none`.
 */
interface WebGpuBackend : AutoCloseable {
    /** `"dawn"`, `"cpu"`, `"none"`, or `"custom:<name>"`. */
    val id: String

    fun hostCallbacks(): ExperimentalHostCallbacks
}

fun interface WebGpuBackendFactory {
    fun create(): WebGpuBackend
}

sealed class WebGpuBackendKind {
    data object None : WebGpuBackendKind()

    data object Dawn : WebGpuBackendKind()

    data class Custom(val id: String) : WebGpuBackendKind()
}
