package io.github.fenriliuguang.wasmtime.android.host.dawn

import io.github.fenriliuguang.wasi.webgpu.experimental.abicm.AbiCmHostBindings
import io.github.fenriliuguang.wasi.webgpu.experimental.host.Extent3D
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.WasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.api.ExperimentalHostCallbacks
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackend

/**
 * [WebGpuBackend] wrapping a [WasiWebGpuHost]. SPI types stay in `runtime-api`;
 * this class is `:host-dawn` only.
 */
class HostWebGpuBackend(
    private val host: WasiWebGpuHost,
    override val id: String,
    private val closeHost: Boolean = true,
) : WebGpuBackend {
    override fun hostCallbacks(): ExperimentalHostCallbacks = ForwardingHostCallbacks(host)

    override fun close() {
        if (closeHost) {
            host.close()
        }
    }
}

private class ForwardingHostCallbacks(
    host: WasiWebGpuHost,
) : ExperimentalHostCallbacks {
    private val bindings = AbiCmHostBindings(host)

    override fun requestAdapter(): Int = bindings.requestAdapter()

    override fun adapterRequestDevice(adapter: Int): Int = bindings.adapterRequestDevice(adapter)

    override fun deviceGetQueue(device: Int): Int = bindings.deviceGetQueue(device)

    override fun createSurfaceFromNativeWindow(windowHandle: Long): Int =
        bindings.createSurfaceFromNativeWindow(windowHandle)

    override fun surfaceConfigure(
        surface: Int,
        device: Int,
        adapter: Int,
        width: Int,
        height: Int,
    ): Int = bindings.surfaceConfigure(surface, device, adapter, width, height)

    override fun surfaceGetCurrentTextureView(surface: Int): Int =
        bindings.surfaceGetCurrentTextureView(surface)

    override fun deviceCreateCommandEncoder(device: Int): Int =
        bindings.deviceCreateCommandEncoder(device)

    override fun deviceCreateBufferDescribed(device: Int, size: Long, usage: Int): Int =
        bindings.deviceCreateBuffer(device, size = size, usage = usage)

    override fun deviceCreateTextureDescribed(
        device: Int,
        width: Int,
        height: Int,
        depth: Int,
        format: Int,
        usage: Int,
    ): Int =
        bindings.deviceCreateTexture(
            device,
            TextureDescriptor(
                size = Extent3D(
                    width = width,
                    height = height,
                    depthOrArrayLayers = depth,
                ),
                format = format,
                usage = usage,
            ),
        )

    override fun bufferMapAsyncDescribed(buffer: Int, mode: Int, offset: Long, size: Long) {
        bindings.bufferMapAsync(buffer, mode, offset, size)
    }

    override fun commandEncoderFinish(encoder: Int): Int = bindings.commandEncoderFinish(encoder)

    override fun queueSubmit1(queue: Int, commandBuffer: Int) {
        bindings.queueSubmit1(queue, commandBuffer)
    }

    override fun surfacePresent(surface: Int) {
        bindings.surfacePresent(surface)
    }

    override fun surfaceUnconfigure(surface: Int) {
        bindings.surfaceUnconfigure(surface)
    }
}
