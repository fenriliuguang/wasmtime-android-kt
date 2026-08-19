package io.github.fenriliuguang.wasmtime.android.host.dawn

import io.github.fenriliuguang.wasi.webgpu.experimental.abicm.AbiCmHostBindings
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ColorTargetState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ComputePipelineDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.FragmentState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.PipelineLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ProgrammableStage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPipelineDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.VertexState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.Extent3D
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuHandle
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureFormat
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPassColorAttachment
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPassDescriptor
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

    override fun deviceCreateCommandEncoderDescribed(device: Int, label: String): Int =
        bindings.deviceCreateCommandEncoder(device, label)

    override fun deviceCreateShaderModuleDescribed(device: Int, code: String): Int =
        bindings.deviceCreateShaderModule(device, code)

    override fun deviceCreateBindGroupLayoutDescribed(
        device: Int,
        binding: Int,
        visibility: Int,
        bufferType: Int,
    ): Int = bindings.deviceCreateBindGroupLayoutDescribed(device, binding, visibility, bufferType)

    override fun deviceCreateBindGroupDescribed(device: Int, layout: Int, label: String): Int =
        bindings.deviceCreateBindGroup(
            device,
            BindGroupDescriptor(
                layout = GpuHandle(layout),
                entries = emptyList(),
                label = label.ifEmpty { null },
            ),
        )

    override fun deviceCreatePipelineLayoutDescribed(
        device: Int,
        bindGroupLayouts: IntArray,
        label: String,
    ): Int =
        bindings.deviceCreatePipelineLayout(
            device,
            PipelineLayoutDescriptor(
                bindGroupLayouts = bindGroupLayouts.map { GpuHandle(it) },
                label = label.ifEmpty { null },
            ),
        )

    override fun deviceCreateComputePipelineDescribed(
        device: Int,
        shader: Int,
        entryPoint: String,
        layout: Int,
        label: String,
    ): Int {
        val module =
            if (shader != 0) {
                GpuHandle(shader)
            } else {
                GpuHandle(bindings.deviceCreateShaderModule(device, COMPUTE_STUB_WGSL))
            }
        val pipelineLayout =
            if (layout != 0) {
                GpuHandle(layout)
            } else {
                GpuHandle(
                    bindings.deviceCreatePipelineLayout(
                        device,
                        PipelineLayoutDescriptor(bindGroupLayouts = emptyList()),
                    ),
                )
            }
        return bindings.deviceCreateComputePipeline(
            device,
            ComputePipelineDescriptor(
                compute = ProgrammableStage(
                    module = module,
                    entryPoint = entryPoint.ifEmpty { "main" },
                ),
                layout = pipelineLayout,
                label = label.ifEmpty { null },
            ),
        )
    }

    override fun deviceCreateRenderPipelineDescribed(
        device: Int,
        vertexShader: Int,
        vertexEntry: String,
        fragmentShader: Int,
        fragmentEntry: String,
        format: Int,
        layout: Int,
        label: String,
    ): Int {
        val vertexModule =
            if (vertexShader != 0) {
                GpuHandle(vertexShader)
            } else {
                GpuHandle(bindings.deviceCreateShaderModule(device, COMPUTE_STUB_WGSL))
            }
        val fragmentModule =
            if (fragmentShader != 0) {
                GpuHandle(fragmentShader)
            } else {
                vertexModule
            }
        val pipelineLayout =
            if (layout != 0) {
                GpuHandle(layout)
            } else {
                GpuHandle(
                    bindings.deviceCreatePipelineLayout(
                        device,
                        PipelineLayoutDescriptor(bindGroupLayouts = emptyList()),
                    ),
                )
            }
        val targetFormat = if (format != 0) format else GpuTextureFormat.RGBA8_UNORM
        return bindings.deviceCreateRenderPipeline(
            device,
            RenderPipelineDescriptor(
                vertex = VertexState(
                    module = vertexModule,
                    entryPoint = vertexEntry.ifEmpty { "vs_main" },
                ),
                fragment = FragmentState(
                    module = fragmentModule,
                    entryPoint = fragmentEntry.ifEmpty { "fs_main" },
                    targets = listOf(ColorTargetState(format = targetFormat)),
                ),
                layout = pipelineLayout,
                label = label.ifEmpty { null },
            ),
        )
    }

    override fun beginComputePassDescribed(
        encoder: Int,
        beginningOfPassWriteIndex: Int,
        endOfPassWriteIndex: Int,
    ): Int = bindings.commandEncoderBeginComputePass(encoder)

    override fun beginRenderPassDescribed(
        encoder: Int,
        view: Int,
        loadOp: Int,
        storeOp: Int,
    ): Int =
        bindings.commandEncoderBeginRenderPass(
            encoder,
            RenderPassDescriptor(
                colorAttachments = listOf(
                    RenderPassColorAttachment(
                        view = GpuHandle(view),
                        loadOp = loadOp,
                        storeOp = storeOp,
                    ),
                ),
            ),
        )

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

    override fun bufferUnmapDescribed(buffer: Int) {
        bindings.bufferUnmap(buffer)
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

    override fun textureWidthDescribed(texture: Int): Int = bindings.textureWidth(texture)

    override fun textureHeightDescribed(texture: Int): Int = bindings.textureHeight(texture)

    override fun textureDepthOrArrayLayersDescribed(texture: Int): Int =
        bindings.textureDepthOrArrayLayers(texture)

    override fun textureMipLevelCountDescribed(texture: Int): Int =
        bindings.textureMipLevelCount(texture)

    override fun textureSampleCountDescribed(texture: Int): Int =
        bindings.textureSampleCount(texture)

    override fun textureDimensionDescribed(texture: Int): Int = bindings.textureDimension(texture)

    override fun textureFormatDescribed(texture: Int): Int = bindings.textureFormat(texture)

    override fun textureUsageDescribed(texture: Int): Int = bindings.textureUsage(texture)

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

    override fun renderPassDrawIndirectDescribed(pass: Int, buffer: Int, offset: Long) {
        bindings.renderPassDrawIndirect(pass, buffer, offset)
    }

    override fun renderPassDrawIndexedIndirectDescribed(pass: Int, buffer: Int, offset: Long) {
        bindings.renderPassDrawIndexedIndirect(pass, buffer, offset)
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

    override fun computePassDispatchWorkgroupsIndirectDescribed(
        pass: Int,
        buffer: Int,
        offset: Long,
    ) {
        bindings.computePassDispatchWorkgroupsIndirect(pass, buffer, offset)
    }

    override fun commandEncoderFinish(encoder: Int): Int = bindings.commandEncoderFinish(encoder)

    override fun commandEncoderFinishDescribed(encoder: Int, label: String): Int =
        bindings.commandEncoderFinish(encoder, label)

    override fun queueSubmit1(queue: Int, commandBuffer: Int) {
        bindings.queueSubmit1(queue, commandBuffer)
    }

    override fun queueSubmitDescribed(queue: Int, commandBuffers: IntArray) {
        bindings.queueSubmit(queue, commandBuffers.toList())
    }

    override fun queueWriteBufferDescribed(
        queue: Int,
        buffer: Int,
        bufferOffset: Long,
        data: ByteArray,
    ) {
        bindings.queueWriteBuffer(queue, buffer, bufferOffset, data)
    }

    override fun queueWriteTextureDescribed(
        queue: Int,
        texture: Int,
        data: ByteArray,
        width: Int,
        height: Int,
        bytesPerRow: Int,
    ) {
        bindings.queueWriteTexture(queue, texture, data, width, height, bytesPerRow)
    }

    override fun surfacePresent(surface: Int) {
        bindings.surfacePresent(surface)
    }

    override fun surfaceUnconfigure(surface: Int) {
        bindings.surfaceUnconfigure(surface)
    }
}

private const val COMPUTE_STUB_WGSL = "@compute @workgroup_size(1) fn main() {}"
