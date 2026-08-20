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

    private class Adapter(
        val vendor: String = "cpu-vendor",
        val architecture: String = "cpu-arch",
        val device: String = "cpu-device",
        val description: String = "cpu-desc",
        val subgroupMinSize: Int = 4,
        val subgroupMaxSize: Int = 128,
        val isFallbackAdapter: Boolean = false,
    )
    private class Device(val adapter: GpuHandle) {
        var errorScopeDepth: Int = 0
        val lostReason: Int = 0
        val lostMessage: String = "cpu-device-lost"
    }
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
        val textureBindingViewDimension: Int = 0,
    )
    private class TextureView
    private class QuerySet(val type: Int, val count: Int)
    private class RenderBundleEncoder
    private class RenderBundle
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
        return handles.insert(ResourceKind.Device, Device(adapter))
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

    override fun deviceCreateQuerySet(device: GpuHandle, type: Int, count: Int): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        require(count > 0) { "query-set count must be positive" }
        return handles.insert(ResourceKind.QuerySet, QuerySet(type = type, count = count))
    }

    override fun deviceCreateRenderBundleEncoder(
        device: GpuHandle,
        colorFormat: Int,
        sampleCount: Int,
    ): GpuHandle {
        handles.get<Device>(device, ResourceKind.Device)
        require(sampleCount > 0) { "bundle-encoder sample count must be positive" }
        return handles.insert(ResourceKind.RenderBundleEncoder, RenderBundleEncoder())
    }

    override fun renderBundleEncoderFinish(encoder: GpuHandle, label: String?): GpuHandle {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        return handles.insert(ResourceKind.RenderBundle, RenderBundle())
    }

    override fun renderBundleEncoderDraw(
        encoder: GpuHandle,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        require(vertexCount >= 0 && instanceCount >= 0)
    }

    override fun renderBundleEncoderDrawIndexed(
        encoder: GpuHandle,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        require(indexCount >= 0 && instanceCount >= 0)
    }

    override fun renderBundleEncoderSetPipeline(encoder: GpuHandle, pipeline: GpuHandle) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        handles.get<RenderPipeline>(pipeline, ResourceKind.RenderPipeline)
    }

    override fun renderBundleEncoderSetVertexBuffer(
        encoder: GpuHandle,
        slot: Int,
        buffer: GpuHandle,
        offset: Long,
        size: Long,
    ) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(slot >= 0 && offset >= 0)
    }

    override fun renderBundleEncoderSetIndexBuffer(
        encoder: GpuHandle,
        buffer: GpuHandle,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(format != 0) { "index format must be non-zero" }
    }

    override fun renderBundleEncoderSetBindGroup(
        encoder: GpuHandle,
        index: Int,
        bindGroup: GpuHandle,
    ) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        handles.get<BindGroup>(bindGroup, ResourceKind.BindGroup)
        require(index >= 0)
    }

    override fun renderBundleEncoderDrawIndirect(
        encoder: GpuHandle,
        buffer: GpuHandle,
        offset: Long,
    ) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(offset >= 0)
    }

    override fun renderBundleEncoderDrawIndexedIndirect(
        encoder: GpuHandle,
        buffer: GpuHandle,
        offset: Long,
    ) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        require(offset >= 0)
    }

    override fun renderBundleEncoderPushDebugGroup(encoder: GpuHandle, label: String) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
    }

    override fun renderBundleEncoderPopDebugGroup(encoder: GpuHandle) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
    }

    override fun renderBundleEncoderInsertDebugMarker(encoder: GpuHandle, label: String) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
    }

    override fun renderBundleEncoderSetImmediates(
        encoder: GpuHandle,
        rangeOffset: Int,
        data: ByteArray,
    ) {
        handles.get<RenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
        require(rangeOffset >= 0)
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

    override fun bufferSetMappedRange(buffer: GpuHandle, offset: Long, data: ByteArray) {
        val buf = handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        if (!buf.mapped) throw HostException.Validation("buffer not mapped")
        require(offset >= 0 && offset + data.size <= buf.size)
        data.copyInto(buf.data, destinationOffset = offset.toInt())
    }

    override fun bufferUnmap(buffer: GpuHandle) {
        val buf = handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        buf.mapped = false
    }

    override fun bufferSize(buffer: GpuHandle): Long =
        handles.get<BufferResource>(buffer, ResourceKind.Buffer).size

    override fun bufferUsage(buffer: GpuHandle): Int =
        handles.get<BufferResource>(buffer, ResourceKind.Buffer).usage

    override fun bufferMapState(buffer: GpuHandle): Int {
        val buf = handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        return if (buf.mapped) GpuBufferMapState.MAPPED else GpuBufferMapState.UNMAPPED
    }

    override fun bufferDestroy(buffer: GpuHandle) {
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
    }

    override fun bufferLabel(buffer: GpuHandle): String {
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
        return ""
    }

    override fun bufferSetLabel(buffer: GpuHandle, label: String) {
        handles.get<BufferResource>(buffer, ResourceKind.Buffer)
    }

    override fun bindGroupLabel(handle: GpuHandle): String {
        handles.get<BindGroup>(handle, ResourceKind.BindGroup)
        return ""
    }

    override fun bindGroupSetLabel(handle: GpuHandle, label: String) {
        handles.get<BindGroup>(handle, ResourceKind.BindGroup)
    }

    override fun bindGroupLayoutLabel(handle: GpuHandle): String {
        handles.get<BindGroupLayout>(handle, ResourceKind.BindGroupLayout)
        return ""
    }

    override fun bindGroupLayoutSetLabel(handle: GpuHandle, label: String) {
        handles.get<BindGroupLayout>(handle, ResourceKind.BindGroupLayout)
    }

    override fun textureLabel(handle: GpuHandle): String {
        handles.get<Texture>(handle, ResourceKind.Texture)
        return ""
    }

    override fun textureSetLabel(handle: GpuHandle, label: String) {
        handles.get<Texture>(handle, ResourceKind.Texture)
    }

    override fun textureViewLabel(handle: GpuHandle): String {
        handles.get<TextureView>(handle, ResourceKind.TextureView)
        return ""
    }

    override fun textureViewSetLabel(handle: GpuHandle, label: String) {
        handles.get<TextureView>(handle, ResourceKind.TextureView)
    }

    override fun samplerLabel(handle: GpuHandle): String {
        handles.get<Sampler>(handle, ResourceKind.Sampler)
        return ""
    }

    override fun samplerSetLabel(handle: GpuHandle, label: String) {
        handles.get<Sampler>(handle, ResourceKind.Sampler)
    }

    override fun shaderModuleLabel(handle: GpuHandle): String {
        handles.get<ShaderModule>(handle, ResourceKind.ShaderModule)
        return ""
    }

    override fun shaderModuleSetLabel(handle: GpuHandle, label: String) {
        handles.get<ShaderModule>(handle, ResourceKind.ShaderModule)
    }

    override fun pipelineLayoutLabel(handle: GpuHandle): String {
        handles.get<PipelineLayout>(handle, ResourceKind.PipelineLayout)
        return ""
    }

    override fun pipelineLayoutSetLabel(handle: GpuHandle, label: String) {
        handles.get<PipelineLayout>(handle, ResourceKind.PipelineLayout)
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

    override fun textureBindingViewDimension(texture: GpuHandle): Int =
        handles.get<Texture>(texture, ResourceKind.Texture).textureBindingViewDimension

    override fun textureDestroy(texture: GpuHandle) {
        handles.get<Texture>(texture, ResourceKind.Texture)
    }

    override fun querySetType(querySet: GpuHandle): Int =
        handles.get<QuerySet>(querySet, ResourceKind.QuerySet).type

    override fun querySetCount(querySet: GpuHandle): Int =
        handles.get<QuerySet>(querySet, ResourceKind.QuerySet).count

    override fun querySetDestroy(querySet: GpuHandle) {
        handles.get<QuerySet>(querySet, ResourceKind.QuerySet)
    }

    override fun commandEncoderResolveQuerySet(
        encoder: GpuHandle,
        querySet: GpuHandle,
        firstQuery: Int,
        queryCount: Int,
        destination: GpuHandle,
        destinationOffset: Long,
    ) {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
        val qs = handles.get<QuerySet>(querySet, ResourceKind.QuerySet)
        handles.get<BufferResource>(destination, ResourceKind.Buffer)
        require(firstQuery >= 0 && queryCount >= 0 && firstQuery + queryCount <= qs.count)
        require(destinationOffset >= 0)
    }

    override fun commandEncoderPushDebugGroup(encoder: GpuHandle, label: String) {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
    }

    override fun commandEncoderPopDebugGroup(encoder: GpuHandle) {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
    }

    override fun commandEncoderInsertDebugMarker(encoder: GpuHandle, label: String) {
        handles.get<CommandEncoder>(encoder, ResourceKind.CommandEncoder)
    }

    override fun adapterValidate(adapter: GpuHandle) {
        handles.get<Adapter>(adapter, ResourceKind.Adapter)
    }

    private fun validateLimitsOwner(adapter: GpuHandle, device: GpuHandle) {
        if (device.raw != 0) {
            handles.get<Device>(device, ResourceKind.Device)
        } else {
            handles.get<Adapter>(adapter, ResourceKind.Adapter)
        }
    }

    override fun supportedLimitsMaxBindGroups(adapter: GpuHandle, device: GpuHandle): Int {
        if (device.raw != 0) {
            handles.get<Device>(device, ResourceKind.Device)
        } else {
            handles.get<Adapter>(adapter, ResourceKind.Adapter)
        }
        return 1
    }

    override fun supportedLimitsMaxBindGroupsPlusVertexBuffers(
        adapter: GpuHandle,
        device: GpuHandle,
    ): Int {
        if (device.raw != 0) {
            handles.get<Device>(device, ResourceKind.Device)
        } else {
            handles.get<Adapter>(adapter, ResourceKind.Adapter)
        }
        return 1
    }

    override fun supportedLimitsMaxBindingsPerBindGroup(adapter: GpuHandle, device: GpuHandle): Int {
        if (device.raw != 0) {
            handles.get<Device>(device, ResourceKind.Device)
        } else {
            handles.get<Adapter>(adapter, ResourceKind.Adapter)
        }
        return 1
    }

    override fun supportedLimitsMaxBufferSize(adapter: GpuHandle, device: GpuHandle): Long {
        if (device.raw != 0) {
            handles.get<Device>(device, ResourceKind.Device)
        } else {
            handles.get<Adapter>(adapter, ResourceKind.Adapter)
        }
        return 1L
    }

    override fun supportedLimitsMaxColorAttachmentBytesPerSample(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxColorAttachments(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxComputeInvocationsPerWorkgroup(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxComputeWorkgroupSizeX(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxComputeWorkgroupSizeY(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxComputeWorkgroupSizeZ(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxComputeWorkgroupsPerDimension(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxComputeWorkgroupStorageSize(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxDynamicStorageBuffersPerPipelineLayout(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxDynamicUniformBuffersPerPipelineLayout(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxImmediateSize(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxInterStageShaderVariables(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxSampledTexturesPerShaderStage(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxSamplersPerShaderStage(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxStorageBufferBindingSize(adapter: GpuHandle, device: GpuHandle): Long {
        validateLimitsOwner(adapter, device)
        return 1L
    }

    override fun supportedLimitsMaxStorageBuffersInFragmentStage(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxStorageBuffersInVertexStage(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxStorageBuffersPerShaderStage(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxStorageTexturesInFragmentStage(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxStorageTexturesInVertexStage(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxStorageTexturesPerShaderStage(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxTextureArrayLayers(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxTextureDimension1D(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxTextureDimension2D(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxTextureDimension3D(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxUniformBufferBindingSize(adapter: GpuHandle, device: GpuHandle): Long {
        validateLimitsOwner(adapter, device)
        return 1L
    }

    override fun supportedLimitsMaxUniformBuffersPerShaderStage(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxVertexAttributes(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxVertexBufferArrayStride(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMaxVertexBuffers(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMinStorageBufferOffsetAlignment(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun supportedLimitsMinUniformBufferOffsetAlignment(adapter: GpuHandle, device: GpuHandle): Int {
        validateLimitsOwner(adapter, device)
        return 1
    }

    override fun adapterInfoSubgroupMinSize(adapter: GpuHandle): Int =
        handles.get<Adapter>(adapter, ResourceKind.Adapter).subgroupMinSize

    override fun adapterInfoSubgroupMaxSize(adapter: GpuHandle): Int =
        handles.get<Adapter>(adapter, ResourceKind.Adapter).subgroupMaxSize

    override fun adapterInfoIsFallbackAdapter(adapter: GpuHandle): Boolean =
        handles.get<Adapter>(adapter, ResourceKind.Adapter).isFallbackAdapter

    override fun adapterInfoVendor(adapter: GpuHandle): String =
        handles.get<Adapter>(adapter, ResourceKind.Adapter).vendor

    override fun adapterInfoArchitecture(adapter: GpuHandle): String =
        handles.get<Adapter>(adapter, ResourceKind.Adapter).architecture

    override fun adapterInfoDevice(adapter: GpuHandle): String =
        handles.get<Adapter>(adapter, ResourceKind.Adapter).device

    override fun adapterInfoDescription(adapter: GpuHandle): String =
        handles.get<Adapter>(adapter, ResourceKind.Adapter).description

    override fun supportedFeaturesHas(adapter: GpuHandle, value: String): Boolean {
        handles.get<Adapter>(adapter, ResourceKind.Adapter)
        return false
    }

    override fun wgslLanguageFeaturesHas(value: String): Boolean = false

    override fun gpuGetPreferredCanvasFormat(): Int = GpuTextureFormat.RGBA8_UNORM

    override fun gpuWgslLanguageFeatures() = Unit

    override fun deviceAdapter(device: GpuHandle): GpuHandle =
        handles.get<Device>(device, ResourceKind.Device).adapter

    override fun deviceValidate(device: GpuHandle) {
        handles.get<Device>(device, ResourceKind.Device)
    }

    override fun deviceDestroy(device: GpuHandle) {
        handles.get<Device>(device, ResourceKind.Device)
    }

    override fun deviceLostInfoReason(device: GpuHandle): Int =
        handles.get<Device>(device, ResourceKind.Device).lostReason

    override fun deviceLostInfoMessage(device: GpuHandle): String =
        handles.get<Device>(device, ResourceKind.Device).lostMessage

    override fun gpuErrorKind(device: GpuHandle): Int {
        handles.get<Device>(device, ResourceKind.Device)
        return 0
    }

    override fun gpuErrorMessage(device: GpuHandle): String {
        handles.get<Device>(device, ResourceKind.Device)
        return "cpu-gpu-error"
    }

    override fun uncapturedErrorEventError(device: GpuHandle) {
        handles.get<Device>(device, ResourceKind.Device)
    }

    override fun devicePushErrorScope(device: GpuHandle, filter: Int) {
        val dev = handles.get<Device>(device, ResourceKind.Device)
        dev.errorScopeDepth += 1
    }

    override fun devicePopErrorScope(device: GpuHandle): Int {
        val dev = handles.get<Device>(device, ResourceKind.Device)
        if (dev.errorScopeDepth > 0) {
            dev.errorScopeDepth -= 1
        }
        return 0
    }

    override fun queueValidate(queue: GpuHandle) {
        handles.get<Queue>(queue, ResourceKind.Queue)
    }

    override fun shaderModuleValidate(shader: GpuHandle) {
        handles.get<ShaderModule>(shader, ResourceKind.ShaderModule)
    }

    override fun compilationMessageType(shader: GpuHandle): Int {
        handles.get<ShaderModule>(shader, ResourceKind.ShaderModule)
        return 0
    }

    override fun compilationMessageLineNum(shader: GpuHandle): Long {
        handles.get<ShaderModule>(shader, ResourceKind.ShaderModule)
        return 42
    }

    override fun compilationMessageLinePos(shader: GpuHandle): Long {
        handles.get<ShaderModule>(shader, ResourceKind.ShaderModule)
        return 7
    }

    override fun compilationMessageOffset(shader: GpuHandle): Long {
        handles.get<ShaderModule>(shader, ResourceKind.ShaderModule)
        return 100
    }

    override fun compilationMessageLength(shader: GpuHandle): Long {
        handles.get<ShaderModule>(shader, ResourceKind.ShaderModule)
        return 256
    }

    override fun compilationMessageMessage(shader: GpuHandle): String {
        handles.get<ShaderModule>(shader, ResourceKind.ShaderModule)
        return "cpu-compilation-message"
    }

    override fun compilationInfoMessagesCount(shader: GpuHandle): Int {
        handles.get<ShaderModule>(shader, ResourceKind.ShaderModule)
        return 1
    }

    override fun renderPipelineGetBindGroupLayout(pipeline: GpuHandle, index: Int): GpuHandle {
        handles.get<RenderPipeline>(pipeline, ResourceKind.RenderPipeline)
        require(index >= 0)
        return handles.insert(ResourceKind.BindGroupLayout, BindGroupLayout())
    }

    override fun computePipelineGetBindGroupLayout(pipeline: GpuHandle, index: Int): GpuHandle {
        handles.get<ComputePipeline>(pipeline, ResourceKind.ComputePipeline)
        require(index >= 0)
        return handles.insert(ResourceKind.BindGroupLayout, BindGroupLayout())
    }

    override fun computePassPushDebugGroup(pass: GpuHandle, label: String) {
        handles.get<ComputePass>(pass, ResourceKind.ComputePassEncoder)
    }

    override fun computePassPopDebugGroup(pass: GpuHandle) {
        handles.get<ComputePass>(pass, ResourceKind.ComputePassEncoder)
    }

    override fun computePassInsertDebugMarker(pass: GpuHandle, label: String) {
        handles.get<ComputePass>(pass, ResourceKind.ComputePassEncoder)
    }

    override fun computePassSetImmediates(pass: GpuHandle, rangeOffset: Int, data: ByteArray) {
        handles.get<ComputePass>(pass, ResourceKind.ComputePassEncoder)
        require(rangeOffset >= 0)
    }

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

    override fun renderPassSetViewport(
        pass: GpuHandle,
        x: Float,
        y: Float,
        width: Float,
        height: Float,
        minDepth: Float,
        maxDepth: Float,
    ) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        require(width >= 0f && height >= 0f)
    }

    override fun renderPassSetScissorRect(pass: GpuHandle, x: Int, y: Int, width: Int, height: Int) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        require(x >= 0 && y >= 0 && width >= 0 && height >= 0)
    }

    override fun renderPassSetBlendConstant(
        pass: GpuHandle,
        r: Double,
        g: Double,
        b: Double,
        a: Double,
    ) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
    }

    override fun renderPassSetStencilReference(pass: GpuHandle, reference: Int) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
    }

    override fun renderPassBeginOcclusionQuery(pass: GpuHandle, queryIndex: Int) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        require(queryIndex >= 0)
    }

    override fun renderPassEndOcclusionQuery(pass: GpuHandle) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
    }

    override fun renderPassExecuteBundles(pass: GpuHandle, bundles: List<GpuHandle>) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
    }

    override fun renderPassPushDebugGroup(pass: GpuHandle, label: String) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
    }

    override fun renderPassPopDebugGroup(pass: GpuHandle) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
    }

    override fun renderPassInsertDebugMarker(pass: GpuHandle, label: String) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
    }

    override fun renderPassSetImmediates(pass: GpuHandle, rangeOffset: Int, data: ByteArray) {
        handles.get<RenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
        require(rangeOffset >= 0)
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
