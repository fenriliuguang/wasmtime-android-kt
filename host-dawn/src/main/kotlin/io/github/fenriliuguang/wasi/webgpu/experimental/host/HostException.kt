package io.github.fenriliuguang.wasi.webgpu.experimental.host

/**
 * Host-side failures as Kotlin exceptions.
 *
 * - **experimental:webgpu-cm** callbacks: still throw → CM trap.
 * - **wasi:webgpu** result-returning methods: map via [HostErrorMapping] into WIT `result` Err
 *   (compliant-world slice F; see `WasiResultCodec` in runtime-wasmtime).
 */
sealed class HostException(message: String, cause: Throwable? = null) :
    RuntimeException(message, cause) {

    class InvalidHandle(val handle: GpuHandle, detail: String) :
        HostException("invalid handle ${handle.raw}: $detail")

    class Unsupported(detail: String) :
        HostException("unsupported in P0 compute subset: $detail")

    class Backend(detail: String, cause: Throwable? = null) :
        HostException(detail, cause)

    class Validation(detail: String) :
        HostException(detail)
}
