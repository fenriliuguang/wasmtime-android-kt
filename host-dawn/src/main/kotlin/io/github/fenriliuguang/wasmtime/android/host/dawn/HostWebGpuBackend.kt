package io.github.fenriliuguang.wasmtime.android.host.dawn

import io.github.fenriliuguang.wasi.webgpu.experimental.abicm.AbiCmHostBindings
import io.github.fenriliuguang.wasi.webgpu.experimental.host.Extent3D
import io.github.fenriliuguang.wasi.webgpu.experimental.host.SamplerDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureViewDescriptor
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

    override fun deviceCreateSamplerDescribed(
        device: Int,
        magFilter: Int,
        minFilter: Int,
        addressModeU: Int,
    ): Int =
        bindings.deviceCreateSampler(
            device,
            SamplerDescriptor(
                magFilter = magFilter,
                minFilter = minFilter,
                addressModeU = addressModeU,
            ),
        )

    override fun textureCreateViewDescribed(
        texture: Int,
        dimension: Int,
        aspect: Int,
    ): Int =
        bindings.textureCreateView(
            texture,
            TextureViewDescriptor(
                dimension = dimension,
                aspect = aspect,
            ),
        )

    override fun commandEncoderCopyBufferToBufferDescribed(
        encoder: Int,
        source: Int,
        sourceOffset: Long,
        destination: Int,
        destinationOffset: Long,
        size: Long,
    ) {
        bindings.commandEncoderCopyBufferToBuffer(
            encoder,
            source,
            sourceOffset,
            destination,
            destinationOffset,
            size,
        )
    }

    override fun commandEncoderClearBufferDescribed(
        encoder: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        bindings.commandEncoderClearBuffer(encoder, buffer, offset, size)
    }

    override fun commandEncoderCopyBufferToTextureDescribed(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        bindings.commandEncoderCopyBufferToTexture(
            encoder,
            source,
            destination,
            width,
            height,
            depth,
        )
    }

    override fun commandEncoderCopyTextureToBufferDescribed(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        bindings.commandEncoderCopyTextureToBuffer(
            encoder,
            source,
            destination,
            width,
            height,
            depth,
        )
    }

    override fun commandEncoderCopyTextureToTextureDescribed(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        bindings.commandEncoderCopyTextureToTexture(
            encoder,
            source,
            destination,
            width,
            height,
            depth,
        )
    }

    override fun renderPassSetPipelineDescribed(pass: Int, pipeline: Int) {
        bindings.renderPassSetPipeline(pass, pipeline)
    }

    override fun renderPassSetBindGroupDescribed(pass: Int, index: Int, bindGroup: Int) {
        bindings.renderPassSetBindGroup(pass, index, bindGroup)
    }

    override fun renderPassSetVertexBufferDescribed(
        pass: Int,
        slot: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        bindings.renderPassSetVertexBuffer(pass, slot, buffer, offset, size)
    }

    override fun renderPassSetIndexBufferDescribed(
        pass: Int,
        buffer: Int,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        bindings.renderPassSetIndexBuffer(pass, buffer, format, offset, size)
    }

    override fun renderPassDrawDescribed(
        pass: Int,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        bindings.renderPassDraw(pass, vertexCount, instanceCount, firstVertex, firstInstance)
    }

    override fun renderPassDrawIndexedDescribed(
        pass: Int,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        bindings.renderPassDrawIndexed(
            pass,
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        )
    }

    override fun renderPassEndDescribed(pass: Int) {
        bindings.renderPassEnd(pass)
    }

    override fun computePassEndDescribed(pass: Int) {
        bindings.computePassEnd(pass)
    }

    override fun computePassSetPipelineDescribed(pass: Int, pipeline: Int) {
        bindings.computePassSetPipeline(pass, pipeline)
    }

    override fun computePassSetBindGroupDescribed(pass: Int, index: Int, bindGroup: Int) {
        bindings.computePassSetBindGroup(pass, index, bindGroup)
    }

    override fun computePassDispatchWorkgroupsDescribed(pass: Int, x: Int, y: Int, z: Int) {
        bindings.computePassDispatchWorkgroups(pass, x, y, z)
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
