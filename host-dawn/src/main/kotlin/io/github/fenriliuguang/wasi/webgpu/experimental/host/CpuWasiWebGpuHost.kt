package io.github.fenriliuguang.wasi.webgpu.experimental.host

import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * In-memory L3 double for desktop/CI. Not a general WGSL runtime.
 *
 * Vector-add dispatch is recognized only when the shader text equals [VectorAddScenario.SHADER].
 */
class CpuWasiWebGpuHost : WasiWebGpuHost {

    private val handles = HandleTable()

    private class Adapter
    private class Device
    private class Queue
    private class ShaderModule(val code: String)
    private class BindGroupLayout
    private class BindGroup(val buffers: List<GpuHandle>)
    private class PipelineLayout
    private class Sampler
    private class Texture(
        var texels: ByteArray? = null,
        val width: Int = 1,
        val height: Int = 1,
        val depthOrArrayLayers: Int = 1,
        val mipLevelCount: Int = 1,
        val sampleCount: Int = 1,
        val dimension: Int = GpuTextureDimension.D2,
        val format: Int = GpuTextureFormat.RGBA8_UNORM,
        val usage: Int = GpuTextureUsage.RENDER_ATTACHMENT,
    )
    private class TextureView
    private class ComputePipeline(val shader: ShaderModule)
    /** Fake Android surface for AbiCm/AbiMvp View↔Texture lifetime tests (not a real window). */
    private class Surface(var configured: Boolean = false)
    /** Handle-only stubs so abi-mvp flat render chain can exercise Cpu Host without Dawn. */
    private class RenderPipeline
    private class RenderPassEncoder

    private class CommandEncoder {
        val copies = ArrayList<CopyOp>()
        var dispatch: DispatchOp? = null
        var finished = false
    }

    private class ComputePass(
        val encoder: CommandEncoder,
        var pipeline: GpuHandle? = null,
        var bindGroup: GpuHandle? = null,
    )

    private class CommandBuffer(
        val copies: List<CopyOp>,
        val dispatch: DispatchOp?,
    )

    private data class CopyOp(
        val source: GpuHandle,
        val sourceOffset: Long,
        val destination: GpuHandle,
        val destinationOffset: Long,
        val size: Long,
    )

    private data class DispatchOp(
        val pipeline: GpuHandle,
        val bindGroup: GpuHandle,
    )

    private class BufferResource(
        val size: Long,
        val usage: Int,
        val data: ByteArray,
        var mapped: Boolean = false,
    )

    override fun requestAdapter(options: RequestAdapterOptions): GpuHandle =
        handles.insert(ResourceKind.Adapter, Adapter())

    override fun adapterRequestDevice(adapter: GpuHandle): GpuHandle {
        handles.get<Adapter>(adapter, ResourceKind.Adapter)
        return handles.insert(ResourceKind.Device, Device())
    }

    override fun deviceGetQueue(device: GpuHandle): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        return handles.insert(ResourceKind.Queue, Queue())
    }

    override fun deviceCreateBuffer(device: GpuHandle, descriptor: BufferDescriptor): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        require(descriptor.size >= 0 && descriptor.size <= Int.MAX_VALUE)
        val buffer = BufferResource(
            size = descriptor.size,
            usage = descriptor.usage,
            data = ByteArray(descriptor.size.toInt()),
            mapped = descriptor.mappedAtCreation,
        )
        return handles.insert(ResourceKind.Buffer, buffer)
    }

    override fun deviceCreateShaderModule(
        device: GpuHandle,
        descriptor: ShaderModuleDescriptor,
    ): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        return handles.insert(ResourceKind.ShaderModule, ShaderModule(descriptor.code))
    }

    override fun deviceCreateBindGroupLayout(
        device: GpuHandle,
        descriptor: BindGroupLayoutDescriptor,
    ): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        for (entry in descriptor.entries) {
            val kinds = listOfNotNull(entry.buffer, entry.sampler, entry.texture)
            if (kinds.isEmpty()) {
                throw HostException.Validation("bind-group-layout entry needs buffer, sampler, or texture")
            }
        }
        return handles.insert(ResourceKind.BindGroupLayout, BindGroupLayout())
    }

    override fun deviceCreateBindGroup(device: GpuHandle, descriptor: BindGroupDescriptor): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        handles.get<BindGroupLayout>(descriptor.layout, ResourceKind.BindGroupLayout)
        val buffers = descriptor.entries
            .sortedBy { it.binding }
            .mapNotNull { entry ->
                when (val resource = entry.resource) {
                    is BindingResource.Buffer -> {
                        handles.get<BufferResource>(resource.binding.buffer, ResourceKind.Buffer)
                        resource.binding.buffer
                    }
                    is BindingResource.Sampler -> {
                        handles.get<Sampler>(resource.sampler, ResourceKind.Sampler)
                        null
                    }
                    is BindingResource.TextureView -> {
                        handles.get<TextureView>(resource.view, ResourceKind.TextureView)
                        null
                    }
                }
            }
        return handles.insert(ResourceKind.BindGroup, BindGroup(buffers))
    }

    override fun deviceCreateTexture(device: GpuHandle, descriptor: TextureDescriptor): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        require(descriptor.size.width > 0 && descriptor.size.height > 0)
        require(descriptor.usage != 0) { "texture usage must be non-zero" }
        return handles.insert(
            ResourceKind.Texture,
            Texture(
                width = descriptor.size.width,
                height = descriptor.size.height,
                depthOrArrayLayers = descriptor.size.depthOrArrayLayers,
                mipLevelCount = descriptor.mipLevelCount,
                sampleCount = descriptor.sampleCount,
                dimension = descriptor.dimension,
                format = descriptor.format,
                usage = descriptor.usage,
            ),
        )
    }

    override fun deviceCreateSampler(device: GpuHandle, descriptor: SamplerDescriptor): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        return handles.insert(ResourceKind.Sampler, Sampler())
    }

    override fun deviceCreatePipelineLayout(
        device: GpuHandle,
        descriptor: PipelineLayoutDescriptor,
    ): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        for (layout in descriptor.bindGroupLayouts) {
            handles.get<BindGroupLayout>(layout, ResourceKind.BindGroupLayout)
        }
        return handles.insert(ResourceKind.PipelineLayout, PipelineLayout())
    }

    override fun deviceCreateComputePipeline(
        device: GpuHandle,
        descriptor: ComputePipelineDescriptor,
    ): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        val layout = descriptor.layout
            ?: throw HostException.Unsupported("auto pipeline layout; pass an explicit pipeline-layout handle")
        handles.get<PipelineLayout>(layout, ResourceKind.PipelineLayout)
        val shader = handles.get<ShaderModule>(descriptor.compute.module, ResourceKind.ShaderModule)
        return handles.insert(ResourceKind.ComputePipeline, ComputePipeline(shader))
    }

    override fun deviceCreateCommandEncoder(
        device: GpuHandle,
        descriptor: CommandEncoderDescriptor,
    ): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        return handles.insert(ResourceKind.CommandEncoder, CommandEncoder())
    }

    override fun commandEncoderBeginComputePass(
        encoder: GpuHandle,
        descriptor: ComputePassDescriptor,
    ): GpuHandle {
        val commandEncoder = handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        if (commandEncoder.finished) {
            throw HostException.Validation("encoder already finished")
        }
        return handles.insert(ResourceKind.ComputePassEncoder, ComputePass(commandEncoder))
    }

    override fun computePassSetPipeline(pass: GpuHandle, pipeline: GpuHandle) {
        val computePass = handles.get<ComputePass>(pass, ResourceKind.ComputePassEncoder)
        handles.get<ComputePipeline>(pipeline, ResourceKind.ComputePipeline)
        computePass.pipeline = pipeline
    }

    override fun computePassSetBindGroup(
        pass: GpuHandle,
        index: Int,
        bindGroup: GpuHandle,
        dynamicOffsets: IntArray,
    ) {
        require(index == 0) { "Cpu host only supports bind group index 0" }
        val computePass = handles.get<ComputePass>(pass, ResourceKind.ComputePassEncoder)
        handles.get<BindGroup>(bindGroup, ResourceKind.BindGroup)
        computePass.bindGroup = bindGroup
    }

    override fun computePassDispatchWorkgroups(
        pass: GpuHandle,
        workgroupCountX: Int,
        workgroupCountY: Int,
        workgroupCountZ: Int,
    ) {
        val computePass = handles.get<ComputePass>(pass, ResourceKind.ComputePassEncoder)
        val pipelineHandle = computePass.pipeline
            ?: throw HostException.Validation("pipeline not set")
        val bindGroupHandle = computePass.bindGroup
            ?: throw HostException.Validation("bind group not set")
        computePass.encoder.dispatch = DispatchOp(pipelineHandle, bindGroupHandle)
    }

    override fun computePassDispatchWorkgroupsIndirect(
        pass: GpuHandle,
        buffer: GpuHandle,
        offset: Long,
    ) {
        val computePass = handles.get<ComputePass>(pass, ResourceKind.ComputePassEncoder)
        val pipelineHandle = computePass.pipeline
            ?: throw HostException.Validation("pipeline not set")
        val bindGroupHandle = computePass.bindGroup
            ?: throw HostException.Validation("bind group not set")
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(offset >= 0) { "indirect offset must be non-negative" }
        computePass.encoder.dispatch = DispatchOp(pipelineHandle, bindGroupHandle)
    }

    override fun computePassEnd(pass: GpuHandle) {
        handles.get<ComputePass>(pass, ResourceKind.ComputePassEncoder)
        handles.drop(pass)
    }

    override fun commandEncoderCopyBufferToBuffer(
        encoder: GpuHandle,
        source: GpuHandle,
        sourceOffset: Long,
        destination: GpuHandle,
        destinationOffset: Long,
        size: Long,
    ) {
        val commandEncoder = handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        handles.get<BufferResource>(source, ResourceKind.Buffer)
        handles.get<BufferResource>(destination, ResourceKind.Buffer)
        commandEncoder.copies += CopyOp(source, sourceOffset, destination, destinationOffset, size)
    }

    override fun commandEncoderClearBuffer(
        encoder: GpuHandle,
        buffer: GpuHandle,
        offset: Long,
        size: Long,
    ) {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
    }

    override fun commandEncoderCopyBufferToTexture(
        encoder: GpuHandle,
        source: GpuHandle,
        destination: GpuHandle,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        handles.get<BufferResource>(source, ResourceKind.Buffer)
        handles.get<Texture>(destination, ResourceKind.Texture)
        require(width > 0 && height > 0 && depth > 0)
    }

    override fun commandEncoderCopyTextureToBuffer(
        encoder: GpuHandle,
        source: GpuHandle,
        destination: GpuHandle,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        handles.get<Texture>(source, ResourceKind.Texture)
        handles.get<BufferResource>(destination, ResourceKind.Buffer)
        require(width > 0 && height > 0 && depth > 0)
    }

    override fun commandEncoderCopyTextureToTexture(
        encoder: GpuHandle,
        source: GpuHandle,
        destination: GpuHandle,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        handles.get<Texture>(source, ResourceKind.Texture)
        handles.get<Texture>(destination, ResourceKind.Texture)
        require(width > 0 && height > 0 && depth > 0)
    }

    @Suppress("UNUSED_PARAMETER")
    override fun commandEncoderFinish(encoder: GpuHandle, label: String?): GpuHandle {
        val commandEncoder = handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        commandEncoder.finished = true
        val commandBuffer = CommandBuffer(commandEncoder.copies.toList(), commandEncoder.dispatch)
        handles.drop(encoder)
        return handles.insert(ResourceKind.CommandBuffer, commandBuffer)
    }

    override fun queueWriteBuffer(
        queue: GpuHandle,
        buffer: GpuHandle,
        bufferOffset: Long,
        data: ByteArray,
    ) {
        handles.get<Queue>(queue, ResourceKind.Queue)
        val buf = handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        val offset = bufferOffset.toInt()
        require(offset >= 0 && offset + data.size <= buf.data.size)
        System.arraycopy(data, 0, buf.data, offset, data.size)
    }

    override fun queueWriteTexture(
        queue: GpuHandle,
        texture: GpuHandle,
        data: ByteArray,
        width: Int,
        height: Int,
        bytesPerRow: Int,
    ) {
        handles.get<Queue>(queue, ResourceKind.Queue)
        val tex = handles.get<Texture>(texture, ResourceKind.Texture)
        require(width > 0 && height > 0) { "invalid write-texture size ${width}x$height" }
        require(bytesPerRow > 0) { "bytesPerRow must be > 0" }
        require(data.size >= bytesPerRow * height) {
            "write-texture data too small: ${data.size} < ${bytesPerRow * height}"
        }
        tex.texels = data.copyOf()
    }

    override fun queueSubmit(queue: GpuHandle, commandBuffers: List<GpuHandle>) {
        handles.get<Queue>(queue, ResourceKind.Queue)
        for (cmdHandle in commandBuffers) {
            val cmd = handles.get<CommandBuffer>(cmdHandle, ResourceKind.CommandBuffer)
            cmd.dispatch?.let { runVectorAddDispatch(it) }
            for (copy in cmd.copies) {
                val src = handles.get<BufferResource>(copy.source, ResourceKind.Buffer)
                val dst = handles.get<BufferResource>(copy.destination, ResourceKind.Buffer)
                System.arraycopy(
                    src.data,
                    copy.sourceOffset.toInt(),
                    dst.data,
                    copy.destinationOffset.toInt(),
                    copy.size.toInt(),
                )
            }
        }
    }

    override fun bufferMapAsync(buffer: GpuHandle, mode: Int, offset: Long, size: Long) {
        val buf = handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(offset >= 0 && offset + size <= buf.size)
        buf.mapped = true
    }

    override fun bufferGetMappedRange(buffer: GpuHandle, offset: Long, size: Long): ByteArray {
        val buf = handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        if (!buf.mapped) throw HostException.Validation("buffer not mapped")
        return buf.data.copyOfRange(offset.toInt(), (offset + size).toInt())
    }

    override fun bufferUnmap(buffer: GpuHandle) {
        val buf = handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        buf.mapped = false
    }

    override fun instanceCreateSurfaceFromAndroidNativeWindow(nativeWindowHandle: Long): GpuHandle {
        require(nativeWindowHandle != 0L) { "window-handle is null" }
        return handles.insert(ResourceKind.Surface, Surface())
    }

    override fun surfaceConfigure(
        surface: GpuHandle,
        device: GpuHandle,
        adapter: GpuHandle,
        width: Int,
        height: Int,
    ): Int {
        require(width > 0 && height > 0) { "invalid surface size ${width}x$height" }
        handles.get<Surface>(surface, ResourceKind.Surface).configured = true
        handles.get<Device>(device, ResourceKind.Device)
        handles.get<Adapter>(adapter, ResourceKind.Adapter)
        return GpuTextureFormat.RGBA8_UNORM
    }

    override fun surfaceUnconfigure(surface: GpuHandle) {
        handles.get<Surface>(surface, ResourceKind.Surface).configured = false
    }

    /**
     * Allocates a fresh fake swapchain [ResourceKind.Texture] per acquire so AbiCm
     * View↔Texture pairing / multi-frame leak tests can run on desktop without Dawn.
     */
    override fun surfaceGetCurrentTexture(surface: GpuHandle): SurfaceTextureResult {
        val s = handles.get<Surface>(surface, ResourceKind.Surface)
        if (!s.configured) {
            throw HostException.Validation("surface not configured")
        }
        val texture = handles.insert(ResourceKind.Texture, Texture())
        return SurfaceTextureResult(SurfaceTextureStatus.SuccessOptimal, texture)
    }

    override fun surfacePresent(surface: GpuHandle) {
        handles.get<Surface>(surface, ResourceKind.Surface)
    }

    /** Test / diagnostics: live handle-table size. */
    fun handleCount(): Int = handles.size()

    /** Test / diagnostics: live handles of [kind]. */
    fun handleCount(kind: ResourceKind): Int = handles.handlesOfKind(kind).size

    override fun deviceCreateRenderPipelineTriangle(
        device: GpuHandle,
        shader: GpuHandle,
        format: Int,
    ): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        handles.get<ShaderModule>(shader, ResourceKind.ShaderModule)
        require(format != 0) { "render pipeline format must be non-zero" }
        return handles.insert(ResourceKind.RenderPipeline, RenderPipeline())
    }

    override fun deviceCreateRenderPipelineTriangleBuffers(
        device: GpuHandle,
        shader: GpuHandle,
        format: Int,
        vertexBuffers: List<VertexBufferLayout>,
    ): GpuHandle = deviceCreateRenderPipelineTriangle(device, shader, format)

    override fun deviceCreateRenderPipeline(
        device: GpuHandle,
        descriptor: RenderPipelineDescriptor,
    ): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        handles.get<ShaderModule>(descriptor.vertex.module, ResourceKind.ShaderModule)
        return handles.insert(ResourceKind.RenderPipeline, RenderPipeline())
    }

    override fun textureCreateView(
        texture: GpuHandle,
        descriptor: TextureViewDescriptor,
    ): GpuHandle {
        handles.get<Texture>(texture, ResourceKind.Texture)
        return handles.insert(ResourceKind.TextureView, TextureView())
    }

    override fun textureWidth(texture: GpuHandle): Int =
        handles.get<Texture>(texture, ResourceKind.Texture).width

    override fun textureHeight(texture: GpuHandle): Int =
        handles.get<Texture>(texture, ResourceKind.Texture).height

    override fun textureDepthOrArrayLayers(texture: GpuHandle): Int =
        handles.get<Texture>(texture, ResourceKind.Texture).depthOrArrayLayers

    override fun textureMipLevelCount(texture: GpuHandle): Int =
        handles.get<Texture>(texture, ResourceKind.Texture).mipLevelCount

    override fun textureSampleCount(texture: GpuHandle): Int =
        handles.get<Texture>(texture, ResourceKind.Texture).sampleCount

    override fun textureDimension(texture: GpuHandle): Int =
        handles.get<Texture>(texture, ResourceKind.Texture).dimension

    override fun textureFormat(texture: GpuHandle): Int =
        handles.get<Texture>(texture, ResourceKind.Texture).format

    override fun textureUsage(texture: GpuHandle): Int =
        handles.get<Texture>(texture, ResourceKind.Texture).usage

    override fun commandEncoderBeginRenderPassClear(
        encoder: GpuHandle,
        view: GpuHandle,
        clearR: Float,
        clearG: Float,
        clearB: Float,
        clearA: Float,
    ): GpuHandle {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        handles.get<TextureView>(view, ResourceKind.TextureView)
        return handles.insert(ResourceKind.RenderPassEncoder, RenderPassEncoder())
    }

    override fun commandEncoderBeginRenderPass(
        encoder: GpuHandle,
        descriptor: RenderPassDescriptor,
    ): GpuHandle {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        descriptor.colorAttachments.forEach {
            handles.get<TextureView>(it.view, ResourceKind.TextureView)
        }
        descriptor.depthStencilAttachment?.let {
            handles.get<TextureView>(it.view, ResourceKind.TextureView)
        }
        return handles.insert(ResourceKind.RenderPassEncoder, RenderPassEncoder())
    }

    override fun renderPassSetPipeline(pass: GpuHandle, pipeline: GpuHandle) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        handles.get<RenderPipeline>(pipeline, ResourceKind.RenderPipeline)
    }

    override fun renderPassSetBindGroup(
        pass: GpuHandle,
        index: Int,
        bindGroup: GpuHandle,
        dynamicOffsets: IntArray,
    ) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        handles.get<BindGroup>(bindGroup, ResourceKind.BindGroup)
        require(index >= 0) { "bind group index must be non-negative" }
    }

    override fun renderPassSetVertexBuffer(
        pass: GpuHandle,
        slot: Int,
        buffer: GpuHandle,
        offset: Long,
        size: Long,
    ) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(slot >= 0) { "vertex buffer slot must be non-negative" }
        require(offset >= 0 && size >= 0) { "vertex buffer range invalid" }
    }

    override fun renderPassSetIndexBuffer(
        pass: GpuHandle,
        buffer: GpuHandle,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(format == GpuIndexFormat.UINT16 || format == GpuIndexFormat.UINT32) {
            "index format must be Uint16 or Uint32"
        }
        require(offset >= 0 && size >= 0) { "index buffer range invalid" }
    }

    override fun renderPassDraw(
        pass: GpuHandle,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        require(vertexCount >= 0) { "vertexCount must be non-negative" }
    }

    override fun renderPassDrawIndexed(
        pass: GpuHandle,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        require(indexCount >= 0) { "indexCount must be non-negative" }
        require(firstIndex >= 0) { "firstIndex must be non-negative" }
    }

    override fun renderPassDrawIndirect(pass: GpuHandle, buffer: GpuHandle, offset: Long) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(offset >= 0) { "indirect offset must be non-negative" }
    }

    override fun renderPassDrawIndexedIndirect(pass: GpuHandle, buffer: GpuHandle, offset: Long) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(offset >= 0) { "indirect offset must be non-negative" }
    }

    override fun renderPassEnd(pass: GpuHandle) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        handles.tryDrop(pass)
    }

    override fun drop(handle: GpuHandle) {
        handles.drop(handle)
    }

    override fun tryDrop(handle: GpuHandle): Boolean = handles.tryDrop(handle) != null

    override fun releaseFrameResources() {
        // Encoder orphans only — see DawnWasiWebGpuHost.releaseFrameResourcesLocked.
        for (
            kind in listOf(
                ResourceKind.CommandBuffer,
                ResourceKind.RenderPassEncoder,
                ResourceKind.ComputePassEncoder,
                ResourceKind.CommandEncoder,
            )
        ) {
            for (handle in handles.handlesOfKind(kind)) {
                tryDrop(handle)
            }
        }
    }

    override fun releaseSurfaces() {
        releaseFrameResources()
        for (
            kind in listOf(
                ResourceKind.TextureView,
                ResourceKind.Texture,
            )
        ) {
            for (handle in handles.handlesOfKind(kind)) {
                tryDrop(handle)
            }
        }
        for (handle in handles.handlesOfKind(ResourceKind.Surface)) {
            runCatching { drop(handle) }
        }
    }

    override fun releaseAllGpuObjects() {
        for (kind in ResourceKind.entries) {
            for (handle in handles.handlesOfKind(kind)) {
                runCatching { drop(handle) }
            }
        }
        handles.clear()
    }

    override fun close() {
        handles.clear()
    }

    private fun runVectorAddDispatch(dispatch: DispatchOp) {
        val pipeline = handles.get<ComputePipeline>(dispatch.pipeline, ResourceKind.ComputePipeline)
        if (pipeline.shader.code != VectorAddScenario.SHADER) {
            throw HostException.Unsupported("Cpu host only executes VectorAddScenario.SHADER")
        }
        val group = handles.get<BindGroup>(dispatch.bindGroup, ResourceKind.BindGroup)
        require(group.buffers.size >= 3) { "vector-add bind group needs 3 buffers" }
        val a = handles.get<BufferResource>(group.buffers[0], ResourceKind.Buffer)
        val b = handles.get<BufferResource>(group.buffers[1], ResourceKind.Buffer)
        val out = handles.get<BufferResource>(group.buffers[2], ResourceKind.Buffer)
        val n = minOf(a.data.size, b.data.size, out.data.size) / 4
        val ab = ByteBuffer.wrap(a.data).order(ByteOrder.LITTLE_ENDIAN)
        val bb = ByteBuffer.wrap(b.data).order(ByteOrder.LITTLE_ENDIAN)
        val ob = ByteBuffer.wrap(out.data).order(ByteOrder.LITTLE_ENDIAN)
        for (i in 0 until n) {
            ob.putFloat(i * 4, ab.getFloat(i * 4) + bb.getFloat(i * 4))
        }
    }
}
