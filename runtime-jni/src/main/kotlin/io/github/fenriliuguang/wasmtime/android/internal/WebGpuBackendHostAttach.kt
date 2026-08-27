package io.github.fenriliuguang.wasmtime.android.internal

/**
 * Internal JNI attach for a [io.github.fenriliuguang.wasmtime.android.api.WebGpuBackend].
 * Not product SPI — apps call `Store.setWebGpuBackend` only.
 */
fun interface WebGpuBackendHostAttach {
    fun attachExperimentalHost(): ExperimentalHostCallbacks
}
