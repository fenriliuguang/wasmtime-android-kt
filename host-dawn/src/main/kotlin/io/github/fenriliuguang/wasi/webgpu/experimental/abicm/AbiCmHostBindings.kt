package io.github.fenriliuguang.wasi.webgpu.experimental.abicm

import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupEntry
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupLayoutEntry
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BufferBinding
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BufferBindingLayout
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BufferBindingType
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BufferDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CommandEncoderDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ComputePipelineDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuHandle
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuQueryType
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuShaderStage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.HostException
import io.github.fenriliuguang.wasi.webgpu.experimental.host.PipelineLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ProgrammableStage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPassDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPipelineDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.SamplerDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ShaderModuleDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.SurfaceTextureStatus
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureViewDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.VertexBufferLayout
import io.github.fenriliuguang.wasi.webgpu.experimental.host.WasiWebGpuHost

/**
 * L1→L2 adapter for experimental CM host imports (typed lists/strings).
 *
 * WIT resource reps arrive as u32 and map 1:1 to L2 [GpuHandle.raw].
 * No Guest linear-memory dependency — buffer bytes arrive as [ByteArray].
 *
 * Frame lifetime (guest-descriptor-cube D): tracks View↔Texture pairs from
 * [surfaceGetCurrentTextureView] because Texture is not a WIT resource and Guest never sees its
 * rep. Pairs are [WasiWebGpuHost.tryDrop]ped on present / next acquire / unconfigure / session
 * handoff. [WasiWebGpuHost.releaseFrameResources] still sweeps encoder orphans.
 *
 * **Still not true WIT destructors** — wasmtime4j `resourceTable` miss skips `.destructor`
 * (see patches/UPSTREAM.md §4). Demo may keep [WasiWebGpuHost.releaseAllGpuObjects] as Session
 * handoff insurance (D2/D3/D6).
 */
class AbiCmHostBindings(
    private val host: WasiWebGpuHost,
) {
    /** view.raw → texture.raw for the current (or last) acquired swapchain frame. */
    private val frameTextureByView = LinkedHashMap<Int, Int>()

    /**
     * Host-side drop for a WIT resource rep (L2 [GpuHandle.raw]).
     * Idempotent. Intended entry for a future rep-only destructor overlay; today Guest `drop`
     * does not reach this path (see UPSTREAM §4).
     */
    fun dropRep(rep: Int): Boolean = host.tryDrop(GpuHandle(rep))

    /** Live View↔Texture pairs awaiting present / next acquire (test / diagnostics). */
    fun trackedFramePairCount(): Int = frameTextureByView.size

    /**
     * Drop tracked swapchain pairs + encoder orphans.
     * Call after Guest `drop-*` / before [WasiWebGpuHost.releaseAllGpuObjects] handoff so the
     * pairing map cannot retain stale reps across Session reuse.
     */
    fun releaseLifetimeSafetyNets() {
        releaseTrackedFrameTextures()
        host.releaseFrameResources()
    }

    fun requestAdapter(): Int = host.requestAdapter().raw

    fun createSurfaceFromNativeWindow(windowHandle: Long): Int =
        host.instanceCreateSurfaceFromAndroidNativeWindow(windowHandle).raw

    fun adapterRequestDevice(adapter: Int): Int =
        host.adapterRequestDevice(GpuHandle(adapter)).raw

    fun deviceGetQueue(device: Int): Int = host.deviceGetQueue(GpuHandle(device)).raw

    fun deviceCreateBuffer(
        device: Int,
        size: Long,
        usage: Int,
        mappedAtCreation: Boolean = false,
        label: String? = null,
    ): Int =
        host.deviceCreateBuffer(
            GpuHandle(device),
            BufferDescriptor(
                size = size,
                usage = usage,
                mappedAtCreation = mappedAtCreation,
                label = label,
            ),
        ).raw

    fun queueWriteBuffer(queue: Int, buffer: Int, offset: Long, data: ByteArray) {
        host.queueWriteBuffer(GpuHandle(queue), GpuHandle(buffer), offset, data)
    }

    fun queueWriteTexture(
        queue: Int,
        texture: Int,
        data: ByteArray,
        width: Int,
        height: Int,
        bytesPerRow: Int,
    ) {
        host.queueWriteTexture(
            GpuHandle(queue),
            GpuHandle(texture),
            data,
            width,
            height,
            bytesPerRow,
        )
    }

    fun deviceCreateShaderModule(device: Int, code: String): Int =
        host.deviceCreateShaderModule(
            GpuHandle(device),
            ShaderModuleDescriptor(code = code),
        ).raw

    fun deviceCreateBindGroupLayout(device: Int, descriptor: BindGroupLayoutDescriptor): Int =
        host.deviceCreateBindGroupLayout(GpuHandle(device), descriptor).raw

    fun deviceCreateBindGroupLayoutDescribed(
        device: Int,
        binding: Int,
        visibility: Int,
        bufferType: Int,
    ): Int {
        val entries = if (bufferType < 0) {
            emptyList()
        } else {
            val type = when (bufferType) {
                1 -> BufferBindingType.Storage
                2 -> BufferBindingType.ReadOnlyStorage
                else -> BufferBindingType.Uniform
            }
            listOf(
                BindGroupLayoutEntry(
                    binding = binding,
                    visibility = visibility,
                    buffer = BufferBindingLayout(type = type),
                ),
            )
        }
        return host.deviceCreateBindGroupLayout(
            GpuHandle(device),
            BindGroupLayoutDescriptor(entries = entries),
        ).raw
    }

    fun deviceCreateBindGroup(device: Int, descriptor: BindGroupDescriptor): Int =
        host.deviceCreateBindGroup(GpuHandle(device), descriptor).raw

    fun deviceCreateTexture(device: Int, descriptor: TextureDescriptor): Int =
        host.deviceCreateTexture(GpuHandle(device), descriptor).raw

    fun deviceCreateQuerySet(
        device: Int,
        type: Int = GpuQueryType.OCCLUSION,
        count: Int = 1,
    ): Int = host.deviceCreateQuerySet(GpuHandle(device), type, count).raw

    fun deviceCreateRenderBundleEncoder(
        device: Int,
        colorFormat: Int,
        sampleCount: Int,
    ): Int = host.deviceCreateRenderBundleEncoder(GpuHandle(device), colorFormat, sampleCount).raw

    fun renderBundleEncoderFinish(encoder: Int, label: String? = null): Int =
        host.renderBundleEncoderFinish(GpuHandle(encoder), label).raw

    fun renderBundleEncoderDraw(
        encoder: Int,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        host.renderBundleEncoderDraw(
            GpuHandle(encoder),
            vertexCount,
            instanceCount,
            firstVertex,
            firstInstance,
        )
    }

    fun renderBundleEncoderDrawIndexed(
        encoder: Int,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        host.renderBundleEncoderDrawIndexed(
            GpuHandle(encoder),
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        )
    }

    fun deviceCreateSampler(device: Int, descriptor: SamplerDescriptor = SamplerDescriptor()): Int =
        host.deviceCreateSampler(GpuHandle(device), descriptor).raw

    fun deviceCreatePipelineLayout(device: Int, descriptor: PipelineLayoutDescriptor): Int =
        host.deviceCreatePipelineLayout(GpuHandle(device), descriptor).raw

    fun deviceCreateComputePipeline(device: Int, descriptor: ComputePipelineDescriptor): Int =
        host.deviceCreateComputePipeline(GpuHandle(device), descriptor).raw

    fun deviceCreateRenderPipeline(device: Int, descriptor: RenderPipelineDescriptor): Int =
        host.deviceCreateRenderPipeline(GpuHandle(device), descriptor).raw

    /** @deprecated Prefer [deviceCreateBindGroupLayout] with a descriptor (slice C). */
    fun deviceCreateBindGroupLayoutStorage3(device: Int): Int {
        val layout = BindGroupLayoutDescriptor(
            entries = listOf(
                storageEntry(0, BufferBindingType.ReadOnlyStorage),
                storageEntry(1, BufferBindingType.ReadOnlyStorage),
                storageEntry(2, BufferBindingType.Storage),
            ),
        )
        return host.deviceCreateBindGroupLayout(GpuHandle(device), layout).raw
    }

    /** @deprecated Prefer [deviceCreateBindGroup] with a descriptor (slice C). */
    fun deviceCreateBindGroup3(device: Int, layout: Int, b0: Int, b1: Int, b2: Int): Int =
        host.deviceCreateBindGroup(
            GpuHandle(device),
            BindGroupDescriptor(
                layout = GpuHandle(layout),
                entries = listOf(
                    BindGroupEntry(0, BufferBinding(GpuHandle(b0))),
                    BindGroupEntry(1, BufferBinding(GpuHandle(b1))),
                    BindGroupEntry(2, BufferBinding(GpuHandle(b2))),
                ),
            ),
        ).raw

    /**
     * Deprecated (slice C/D): builds a 1×BGL pipeline-layout then create-compute-pipeline.
     * Prefer explicit [deviceCreatePipelineLayout] + [deviceCreateComputePipeline].
     */
    fun deviceCreateComputePipelineBgl(
        device: Int,
        layout: Int,
        shader: Int,
        entryPoint: String,
    ): Int {
        val pipelineLayout = host.deviceCreatePipelineLayout(
            GpuHandle(device),
            PipelineLayoutDescriptor(bindGroupLayouts = listOf(GpuHandle(layout))),
        )
        return host.deviceCreateComputePipeline(
            GpuHandle(device),
            ComputePipelineDescriptor(
                layout = pipelineLayout,
                compute = ProgrammableStage(module = GpuHandle(shader), entryPoint = entryPoint),
            ),
        ).raw
    }

    fun deviceCreateRenderPipelineTriangle(device: Int, shader: Int, format: Int): Int =
        host.deviceCreateRenderPipelineTriangle(
            GpuHandle(device),
            GpuHandle(shader),
            format,
        ).raw

    fun deviceCreateRenderPipelineTriangleBuffers(
        device: Int,
        shader: Int,
        format: Int,
        vertexBuffers: List<VertexBufferLayout>,
    ): Int =
        host.deviceCreateRenderPipelineTriangleBuffers(
            GpuHandle(device),
            GpuHandle(shader),
            format,
            vertexBuffers,
        ).raw

    fun deviceCreateCommandEncoder(device: Int): Int =
        host.deviceCreateCommandEncoder(GpuHandle(device)).raw

    fun deviceCreateCommandEncoder(device: Int, label: String): Int =
        host.deviceCreateCommandEncoder(
            GpuHandle(device),
            CommandEncoderDescriptor(label = label.ifEmpty { null }),
        ).raw

    fun textureCreateView(
        texture: Int,
        descriptor: TextureViewDescriptor = TextureViewDescriptor(),
    ): Int = host.textureCreateView(GpuHandle(texture), descriptor).raw

    fun textureWidth(texture: Int): Int = host.textureWidth(GpuHandle(texture))

    fun textureHeight(texture: Int): Int = host.textureHeight(GpuHandle(texture))

    fun textureDepthOrArrayLayers(texture: Int): Int =
        host.textureDepthOrArrayLayers(GpuHandle(texture))

    fun textureMipLevelCount(texture: Int): Int = host.textureMipLevelCount(GpuHandle(texture))

    fun textureSampleCount(texture: Int): Int = host.textureSampleCount(GpuHandle(texture))

    fun textureDimension(texture: Int): Int = host.textureDimension(GpuHandle(texture))

    fun textureFormat(texture: Int): Int = host.textureFormat(GpuHandle(texture))

    fun textureUsage(texture: Int): Int = host.textureUsage(GpuHandle(texture))

    fun textureBindingViewDimension(texture: Int): Int =
        host.textureBindingViewDimension(GpuHandle(texture))

    fun textureDestroy(texture: Int) {
        host.textureDestroy(GpuHandle(texture))
    }

    fun surfaceConfigure(surface: Int, device: Int, adapter: Int, width: Int, height: Int): Int =
        host.surfaceConfigure(
            GpuHandle(surface),
            GpuHandle(device),
            GpuHandle(adapter),
            width,
            height,
        )

    fun surfaceGetCurrentTextureView(surface: Int): Int {
        // Drop prior View↔Texture pair (and sweep encoder orphans) before acquire (D5).
        releaseTrackedFrameTextures()
        host.releaseFrameResources()
        val result = host.surfaceGetCurrentTexture(GpuHandle(surface))
        if (
            result.status != SurfaceTextureStatus.SuccessOptimal &&
            result.status != SurfaceTextureStatus.SuccessSuboptimal
        ) {
            throw HostException.Validation("surface get-current-texture status=${result.status}")
        }
        val texture = result.texture
            ?: throw HostException.Validation("surface get-current-texture returned null texture")
        // Guest only receives the view rep; Texture is Host-private — pair for present/drop.
        val view = host.textureCreateView(texture)
        frameTextureByView[view.raw] = texture.raw
        return view.raw
    }

    fun surfacePresent(surface: Int) {
        host.surfacePresent(GpuHandle(surface))
        // Return swapchain buffers via paired drop first; sweep covers leftover encoders.
        releaseTrackedFrameTextures()
        host.releaseFrameResources()
    }

    private fun releaseTrackedFrameTextures() {
        if (frameTextureByView.isEmpty()) return
        val pairs = frameTextureByView.entries.toList()
        frameTextureByView.clear()
        for ((viewRaw, textureRaw) in pairs) {
            host.tryDrop(GpuHandle(viewRaw))
            host.tryDrop(GpuHandle(textureRaw))
        }
    }

    fun surfaceUnconfigure(surface: Int) {
        // Guest may unconfigure while a View↔Texture pair is still tracked (failed present, etc.).
        releaseTrackedFrameTextures()
        host.surfaceUnconfigure(GpuHandle(surface))
    }

    fun commandEncoderBeginComputePass(encoder: Int): Int =
        host.commandEncoderBeginComputePass(GpuHandle(encoder)).raw

    fun commandEncoderBeginRenderPassClear(
        encoder: Int,
        view: Int,
        r: Float,
        g: Float,
        b: Float,
        a: Float,
    ): Int =
        host.commandEncoderBeginRenderPassClear(
            GpuHandle(encoder),
            GpuHandle(view),
            r,
            g,
            b,
            a,
        ).raw

    fun commandEncoderBeginRenderPass(encoder: Int, descriptor: RenderPassDescriptor): Int =
        host.commandEncoderBeginRenderPass(GpuHandle(encoder), descriptor).raw

    fun computePassSetPipeline(pass: Int, pipeline: Int) {
        host.computePassSetPipeline(GpuHandle(pass), GpuHandle(pipeline))
    }

    fun computePassSetBindGroup(pass: Int, index: Int, bindGroup: Int) {
        host.computePassSetBindGroup(GpuHandle(pass), index, GpuHandle(bindGroup))
    }

    fun computePassDispatchWorkgroups(pass: Int, x: Int, y: Int, z: Int) {
        host.computePassDispatchWorkgroups(GpuHandle(pass), x, y, z)
    }

    fun computePassDispatchWorkgroupsIndirect(pass: Int, buffer: Int, offset: Long) {
        host.computePassDispatchWorkgroupsIndirect(GpuHandle(pass), GpuHandle(buffer), offset)
    }

    fun computePassEnd(pass: Int) {
        host.computePassEnd(GpuHandle(pass))
    }

    fun renderPassSetPipeline(pass: Int, pipeline: Int) {
        host.renderPassSetPipeline(GpuHandle(pass), GpuHandle(pipeline))
    }

    fun renderPassSetBindGroup(pass: Int, index: Int, bindGroup: Int) {
        host.renderPassSetBindGroup(GpuHandle(pass), index, GpuHandle(bindGroup))
    }

    fun renderPassSetVertexBuffer(pass: Int, slot: Int, buffer: Int, offset: Long, size: Long) {
        host.renderPassSetVertexBuffer(
            GpuHandle(pass),
            slot,
            GpuHandle(buffer),
            offset,
            size,
        )
    }

    fun renderPassSetIndexBuffer(
        pass: Int,
        buffer: Int,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        host.renderPassSetIndexBuffer(
            GpuHandle(pass),
            GpuHandle(buffer),
            format,
            offset,
            size,
        )
    }

    fun renderPassDraw(
        pass: Int,
        vertexCount: Int,
        instanceCount: Int = 1,
        firstVertex: Int = 0,
        firstInstance: Int = 0,
    ) {
        host.renderPassDraw(
            GpuHandle(pass),
            vertexCount,
            instanceCount,
            firstVertex,
            firstInstance,
        )
    }

    fun renderPassDrawIndexed(
        pass: Int,
        indexCount: Int,
        instanceCount: Int = 1,
        firstIndex: Int = 0,
        baseVertex: Int = 0,
        firstInstance: Int = 0,
    ) {
        host.renderPassDrawIndexed(
            GpuHandle(pass),
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        )
    }

    fun renderPassDrawIndirect(pass: Int, buffer: Int, offset: Long) {
        host.renderPassDrawIndirect(GpuHandle(pass), GpuHandle(buffer), offset)
    }

    fun renderPassDrawIndexedIndirect(pass: Int, buffer: Int, offset: Long) {
        host.renderPassDrawIndexedIndirect(GpuHandle(pass), GpuHandle(buffer), offset)
    }

    fun renderPassEnd(pass: Int) {
        host.renderPassEnd(GpuHandle(pass))
    }

    fun renderPassSetViewport(
        pass: Int,
        x: Float,
        y: Float,
        width: Float,
        height: Float,
        minDepth: Float,
        maxDepth: Float,
    ) {
        host.renderPassSetViewport(GpuHandle(pass), x, y, width, height, minDepth, maxDepth)
    }

    fun renderPassSetScissorRect(pass: Int, x: Int, y: Int, width: Int, height: Int) {
        host.renderPassSetScissorRect(GpuHandle(pass), x, y, width, height)
    }

    fun renderPassSetBlendConstant(pass: Int, r: Double, g: Double, b: Double, a: Double) {
        host.renderPassSetBlendConstant(GpuHandle(pass), r, g, b, a)
    }

    fun renderPassSetStencilReference(pass: Int, reference: Int) {
        host.renderPassSetStencilReference(GpuHandle(pass), reference)
    }

    fun renderPassBeginOcclusionQuery(pass: Int, queryIndex: Int) {
        host.renderPassBeginOcclusionQuery(GpuHandle(pass), queryIndex)
    }

    fun renderPassEndOcclusionQuery(pass: Int) {
        host.renderPassEndOcclusionQuery(GpuHandle(pass))
    }

    fun renderPassExecuteBundles(pass: Int, bundles: IntArray) {
        host.renderPassExecuteBundles(
            GpuHandle(pass),
            bundles.filter { it != 0 }.map { GpuHandle(it) },
        )
    }

    fun renderPassPushDebugGroup(pass: Int, label: String) {
        host.renderPassPushDebugGroup(GpuHandle(pass), label)
    }

    fun renderPassPopDebugGroup(pass: Int) {
        host.renderPassPopDebugGroup(GpuHandle(pass))
    }

    fun renderPassInsertDebugMarker(pass: Int, label: String) {
        host.renderPassInsertDebugMarker(GpuHandle(pass), label)
    }

    fun renderPassSetImmediates(pass: Int, rangeOffset: Int, data: ByteArray) {
        host.renderPassSetImmediates(GpuHandle(pass), rangeOffset, data)
    }

    fun commandEncoderCopyBufferToBuffer(
        encoder: Int,
        source: Int,
        sourceOffset: Long,
        destination: Int,
        destinationOffset: Long,
        size: Long,
    ) {
        host.commandEncoderCopyBufferToBuffer(
            GpuHandle(encoder),
            GpuHandle(source),
            sourceOffset,
            GpuHandle(destination),
            destinationOffset,
            size,
        )
    }

    fun commandEncoderClearBuffer(
        encoder: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        host.commandEncoderClearBuffer(
            GpuHandle(encoder),
            GpuHandle(buffer),
            offset,
            size,
        )
    }

    fun commandEncoderCopyBufferToTexture(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        host.commandEncoderCopyBufferToTexture(
            GpuHandle(encoder),
            GpuHandle(source),
            GpuHandle(destination),
            width,
            height,
            depth,
        )
    }

    fun commandEncoderCopyTextureToBuffer(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        host.commandEncoderCopyTextureToBuffer(
            GpuHandle(encoder),
            GpuHandle(source),
            GpuHandle(destination),
            width,
            height,
            depth,
        )
    }

    fun commandEncoderCopyTextureToTexture(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        host.commandEncoderCopyTextureToTexture(
            GpuHandle(encoder),
            GpuHandle(source),
            GpuHandle(destination),
            width,
            height,
            depth,
        )
    }

    fun commandEncoderFinish(encoder: Int): Int =
        host.commandEncoderFinish(GpuHandle(encoder)).raw

    fun commandEncoderFinish(encoder: Int, label: String): Int =
        host.commandEncoderFinish(GpuHandle(encoder), label.ifEmpty { null }).raw

    fun queueSubmit(queue: Int, commandBuffers: List<Int>) {
        host.queueSubmit(GpuHandle(queue), commandBuffers.map { GpuHandle(it) })
    }

    /** @deprecated Prefer [queueSubmit] (slice C). */
    fun queueSubmit1(queue: Int, commandBuffer: Int) {
        host.queueSubmit(GpuHandle(queue), listOf(GpuHandle(commandBuffer)))
    }

    fun renderBundleEncoderSetPipeline(encoder: Int, pipeline: Int) {
        host.renderBundleEncoderSetPipeline(GpuHandle(encoder), GpuHandle(pipeline))
    }

    fun renderBundleEncoderSetVertexBuffer(
        encoder: Int,
        slot: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        host.renderBundleEncoderSetVertexBuffer(
            GpuHandle(encoder),
            slot,
            GpuHandle(buffer),
            offset,
            size,
        )
    }

    fun renderBundleEncoderSetIndexBuffer(
        encoder: Int,
        buffer: Int,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        host.renderBundleEncoderSetIndexBuffer(
            GpuHandle(encoder),
            GpuHandle(buffer),
            format,
            offset,
            size,
        )
    }

    fun renderBundleEncoderSetBindGroup(encoder: Int, index: Int, bindGroup: Int) {
        host.renderBundleEncoderSetBindGroup(GpuHandle(encoder), index, GpuHandle(bindGroup))
    }

    fun renderBundleEncoderDrawIndirect(encoder: Int, buffer: Int, offset: Long) {
        host.renderBundleEncoderDrawIndirect(GpuHandle(encoder), GpuHandle(buffer), offset)
    }

    fun renderBundleEncoderDrawIndexedIndirect(encoder: Int, buffer: Int, offset: Long) {
        host.renderBundleEncoderDrawIndexedIndirect(GpuHandle(encoder), GpuHandle(buffer), offset)
    }

    fun renderBundleEncoderPushDebugGroup(encoder: Int, label: String) {
        host.renderBundleEncoderPushDebugGroup(GpuHandle(encoder), label)
    }

    fun renderBundleEncoderPopDebugGroup(encoder: Int) {
        host.renderBundleEncoderPopDebugGroup(GpuHandle(encoder))
    }

    fun renderBundleEncoderInsertDebugMarker(encoder: Int, label: String) {
        host.renderBundleEncoderInsertDebugMarker(GpuHandle(encoder), label)
    }

    fun renderBundleEncoderSetImmediates(encoder: Int, rangeOffset: Int, data: ByteArray) {
        host.renderBundleEncoderSetImmediates(GpuHandle(encoder), rangeOffset, data)
    }

    fun bufferMapAsync(buffer: Int, mode: Int, offset: Long, size: Long) {
        host.bufferMapAsync(GpuHandle(buffer), mode, offset, size)
    }

    fun bufferGetMappedRange(buffer: Int, offset: Long, size: Long): ByteArray =
        host.bufferGetMappedRange(GpuHandle(buffer), offset, size)

    fun bufferSetMappedRange(buffer: Int, offset: Long, data: ByteArray) {
        host.bufferSetMappedRange(GpuHandle(buffer), offset, data)
    }

    fun bufferUnmap(buffer: Int) {
        host.bufferUnmap(GpuHandle(buffer))
    }

    fun bufferSize(buffer: Int): Long = host.bufferSize(GpuHandle(buffer))

    fun bufferUsage(buffer: Int): Int = host.bufferUsage(GpuHandle(buffer))

    fun bufferMapState(buffer: Int): Int = host.bufferMapState(GpuHandle(buffer))

    fun bufferDestroy(buffer: Int) {
        host.bufferDestroy(GpuHandle(buffer))
    }

    fun bufferLabel(buffer: Int): String = host.bufferLabel(GpuHandle(buffer))

    fun bufferSetLabel(buffer: Int, label: String) {
        host.bufferSetLabel(GpuHandle(buffer), label)
    }

    fun bindGroupLabel(handle: Int): String = host.bindGroupLabel(GpuHandle(handle))

    fun bindGroupSetLabel(handle: Int, label: String) {
        host.bindGroupSetLabel(GpuHandle(handle), label)
    }

    fun bindGroupLayoutLabel(handle: Int): String = host.bindGroupLayoutLabel(GpuHandle(handle))

    fun bindGroupLayoutSetLabel(handle: Int, label: String) {
        host.bindGroupLayoutSetLabel(GpuHandle(handle), label)
    }

    fun textureLabel(handle: Int): String = host.textureLabel(GpuHandle(handle))

    fun textureSetLabel(handle: Int, label: String) {
        host.textureSetLabel(GpuHandle(handle), label)
    }

    fun textureViewLabel(handle: Int): String = host.textureViewLabel(GpuHandle(handle))

    fun textureViewSetLabel(handle: Int, label: String) {
        host.textureViewSetLabel(GpuHandle(handle), label)
    }

    fun samplerLabel(handle: Int): String = host.samplerLabel(GpuHandle(handle))

    fun samplerSetLabel(handle: Int, label: String) {
        host.samplerSetLabel(GpuHandle(handle), label)
    }

    fun shaderModuleLabel(handle: Int): String = host.shaderModuleLabel(GpuHandle(handle))

    fun shaderModuleSetLabel(handle: Int, label: String) {
        host.shaderModuleSetLabel(GpuHandle(handle), label)
    }

    fun pipelineLayoutLabel(handle: Int): String = host.pipelineLayoutLabel(GpuHandle(handle))

    fun pipelineLayoutSetLabel(handle: Int, label: String) {
        host.pipelineLayoutSetLabel(GpuHandle(handle), label)
    }

    fun querySetLabel(handle: Int): String = host.querySetLabel(GpuHandle(handle))

    fun querySetSetLabel(handle: Int, label: String) {
        host.querySetSetLabel(GpuHandle(handle), label)
    }

    fun deviceLabel(handle: Int): String = host.deviceLabel(GpuHandle(handle))

    fun deviceSetLabel(handle: Int, label: String) {
        host.deviceSetLabel(GpuHandle(handle), label)
    }

    fun queueLabel(handle: Int): String = host.queueLabel(GpuHandle(handle))

    fun queueSetLabel(handle: Int, label: String) {
        host.queueSetLabel(GpuHandle(handle), label)
    }

    fun commandEncoderLabel(handle: Int): String = host.commandEncoderLabel(GpuHandle(handle))

    fun commandEncoderSetLabel(handle: Int, label: String) {
        host.commandEncoderSetLabel(GpuHandle(handle), label)
    }

    fun commandBufferLabel(handle: Int): String = host.commandBufferLabel(GpuHandle(handle))

    fun commandBufferSetLabel(handle: Int, label: String) {
        host.commandBufferSetLabel(GpuHandle(handle), label)
    }

    fun computePassEncoderLabel(handle: Int): String = host.computePassEncoderLabel(GpuHandle(handle))

    fun computePassEncoderSetLabel(handle: Int, label: String) {
        host.computePassEncoderSetLabel(GpuHandle(handle), label)
    }

    fun computePipelineLabel(handle: Int): String = host.computePipelineLabel(GpuHandle(handle))

    fun computePipelineSetLabel(handle: Int, label: String) {
        host.computePipelineSetLabel(GpuHandle(handle), label)
    }

    fun renderBundleEncoderLabel(handle: Int): String = host.renderBundleEncoderLabel(GpuHandle(handle))

    fun renderBundleEncoderSetLabel(handle: Int, label: String) {
        host.renderBundleEncoderSetLabel(GpuHandle(handle), label)
    }

    fun renderBundleLabel(handle: Int): String = host.renderBundleLabel(GpuHandle(handle))

    fun renderBundleSetLabel(handle: Int, label: String) {
        host.renderBundleSetLabel(GpuHandle(handle), label)
    }

    fun querySetType(querySet: Int): Int = host.querySetType(GpuHandle(querySet))

    fun querySetCount(querySet: Int): Int = host.querySetCount(GpuHandle(querySet))

    fun querySetDestroy(querySet: Int) {
        host.querySetDestroy(GpuHandle(querySet))
    }

    fun commandEncoderResolveQuerySet(
        encoder: Int,
        querySet: Int,
        firstQuery: Int,
        queryCount: Int,
        destination: Int,
        destinationOffset: Long,
    ) {
        host.commandEncoderResolveQuerySet(
            GpuHandle(encoder),
            GpuHandle(querySet),
            firstQuery,
            queryCount,
            GpuHandle(destination),
            destinationOffset,
        )
    }

    fun commandEncoderPushDebugGroup(encoder: Int, label: String) {
        host.commandEncoderPushDebugGroup(GpuHandle(encoder), label)
    }

    fun commandEncoderPopDebugGroup(encoder: Int) {
        host.commandEncoderPopDebugGroup(GpuHandle(encoder))
    }

    fun commandEncoderInsertDebugMarker(encoder: Int, label: String) {
        host.commandEncoderInsertDebugMarker(GpuHandle(encoder), label)
    }

    fun adapterValidate(adapter: Int) {
        host.adapterValidate(GpuHandle(adapter))
    }

    fun supportedLimitsMaxBindGroups(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxBindGroups(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxBindGroupsPlusVertexBuffers(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxBindGroupsPlusVertexBuffers(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxBindingsPerBindGroup(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxBindingsPerBindGroup(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxBufferSize(adapter: Int, device: Int): Long =
        host.supportedLimitsMaxBufferSize(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxColorAttachmentBytesPerSample(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxColorAttachmentBytesPerSample(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxColorAttachments(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxColorAttachments(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxComputeInvocationsPerWorkgroup(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxComputeInvocationsPerWorkgroup(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxComputeWorkgroupSizeX(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxComputeWorkgroupSizeX(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxComputeWorkgroupSizeY(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxComputeWorkgroupSizeY(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxComputeWorkgroupSizeZ(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxComputeWorkgroupSizeZ(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxComputeWorkgroupsPerDimension(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxComputeWorkgroupsPerDimension(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxComputeWorkgroupStorageSize(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxComputeWorkgroupStorageSize(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxDynamicStorageBuffersPerPipelineLayout(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxDynamicStorageBuffersPerPipelineLayout(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxDynamicUniformBuffersPerPipelineLayout(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxDynamicUniformBuffersPerPipelineLayout(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxImmediateSize(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxImmediateSize(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxInterStageShaderVariables(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxInterStageShaderVariables(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxSampledTexturesPerShaderStage(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxSampledTexturesPerShaderStage(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxSamplersPerShaderStage(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxSamplersPerShaderStage(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxStorageBufferBindingSize(adapter: Int, device: Int): Long =
        host.supportedLimitsMaxStorageBufferBindingSize(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxStorageBuffersInFragmentStage(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxStorageBuffersInFragmentStage(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxStorageBuffersInVertexStage(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxStorageBuffersInVertexStage(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxStorageBuffersPerShaderStage(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxStorageBuffersPerShaderStage(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxStorageTexturesInFragmentStage(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxStorageTexturesInFragmentStage(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxStorageTexturesInVertexStage(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxStorageTexturesInVertexStage(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxStorageTexturesPerShaderStage(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxStorageTexturesPerShaderStage(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxTextureArrayLayers(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxTextureArrayLayers(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxTextureDimension1D(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxTextureDimension1D(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxTextureDimension2D(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxTextureDimension2D(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxTextureDimension3D(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxTextureDimension3D(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxUniformBufferBindingSize(adapter: Int, device: Int): Long =
        host.supportedLimitsMaxUniformBufferBindingSize(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxUniformBuffersPerShaderStage(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxUniformBuffersPerShaderStage(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxVertexAttributes(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxVertexAttributes(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxVertexBufferArrayStride(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxVertexBufferArrayStride(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMaxVertexBuffers(adapter: Int, device: Int): Int =
        host.supportedLimitsMaxVertexBuffers(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMinStorageBufferOffsetAlignment(adapter: Int, device: Int): Int =
        host.supportedLimitsMinStorageBufferOffsetAlignment(GpuHandle(adapter), GpuHandle(device))

    fun supportedLimitsMinUniformBufferOffsetAlignment(adapter: Int, device: Int): Int =
        host.supportedLimitsMinUniformBufferOffsetAlignment(GpuHandle(adapter), GpuHandle(device))

    fun adapterInfoSubgroupMinSize(adapter: Int): Int =
        host.adapterInfoSubgroupMinSize(GpuHandle(adapter))

    fun adapterInfoSubgroupMaxSize(adapter: Int): Int =
        host.adapterInfoSubgroupMaxSize(GpuHandle(adapter))

    fun adapterInfoIsFallbackAdapter(adapter: Int): Boolean =
        host.adapterInfoIsFallbackAdapter(GpuHandle(adapter))

    fun adapterInfoVendor(adapter: Int): String =
        host.adapterInfoVendor(GpuHandle(adapter))

    fun adapterInfoArchitecture(adapter: Int): String =
        host.adapterInfoArchitecture(GpuHandle(adapter))

    fun adapterInfoDevice(adapter: Int): String = host.adapterInfoDevice(GpuHandle(adapter))

    fun adapterInfoDescription(adapter: Int): String =
        host.adapterInfoDescription(GpuHandle(adapter))

    fun supportedFeaturesHas(adapter: Int, value: String): Boolean =
        host.supportedFeaturesHas(GpuHandle(adapter), value)

    fun wgslLanguageFeaturesHas(value: String): Boolean = host.wgslLanguageFeaturesHas(value)

    fun gpuGetPreferredCanvasFormat(): Int = host.gpuGetPreferredCanvasFormat()

    fun gpuWgslLanguageFeatures() {
        host.gpuWgslLanguageFeatures()
    }

    fun deviceAdapter(device: Int): Int = host.deviceAdapter(GpuHandle(device)).raw

    fun deviceValidate(device: Int) {
        host.deviceValidate(GpuHandle(device))
    }

    fun deviceDestroy(device: Int) {
        host.deviceDestroy(GpuHandle(device))
    }

    fun deviceLostInfoReason(device: Int): Int =
        host.deviceLostInfoReason(GpuHandle(device))

    fun deviceLostInfoMessage(device: Int): String =
        host.deviceLostInfoMessage(GpuHandle(device))

    fun gpuErrorKind(device: Int): Int = host.gpuErrorKind(GpuHandle(device))

    fun gpuErrorMessage(device: Int): String = host.gpuErrorMessage(GpuHandle(device))

    fun uncapturedErrorEventError(device: Int) {
        host.uncapturedErrorEventError(GpuHandle(device))
    }

    fun devicePushErrorScope(device: Int, filter: Int) {
        host.devicePushErrorScope(GpuHandle(device), filter)
    }

    fun devicePopErrorScope(device: Int): Int = host.devicePopErrorScope(GpuHandle(device))

    fun queueValidate(queue: Int) {
        host.queueValidate(GpuHandle(queue))
    }

    fun shaderModuleValidate(shader: Int) {
        host.shaderModuleValidate(GpuHandle(shader))
    }

    fun compilationMessageType(shader: Int): Int =
        host.compilationMessageType(GpuHandle(shader))

    fun compilationMessageLineNum(shader: Int): Long =
        host.compilationMessageLineNum(GpuHandle(shader))

    fun compilationMessageLinePos(shader: Int): Long =
        host.compilationMessageLinePos(GpuHandle(shader))

    fun compilationMessageOffset(shader: Int): Long =
        host.compilationMessageOffset(GpuHandle(shader))

    fun compilationMessageLength(shader: Int): Long =
        host.compilationMessageLength(GpuHandle(shader))

    fun compilationMessageMessage(shader: Int): String =
        host.compilationMessageMessage(GpuHandle(shader))

    fun compilationInfoMessagesCount(shader: Int): Int =
        host.compilationInfoMessagesCount(GpuHandle(shader))

    fun renderPipelineGetBindGroupLayout(pipeline: Int, index: Int): Int =
        host.renderPipelineGetBindGroupLayout(GpuHandle(pipeline), index).raw

    fun computePipelineGetBindGroupLayout(pipeline: Int, index: Int): Int =
        host.computePipelineGetBindGroupLayout(GpuHandle(pipeline), index).raw

    fun computePassPushDebugGroup(pass: Int, label: String) {
        host.computePassPushDebugGroup(GpuHandle(pass), label)
    }

    fun computePassPopDebugGroup(pass: Int) {
        host.computePassPopDebugGroup(GpuHandle(pass))
    }

    fun computePassInsertDebugMarker(pass: Int, label: String) {
        host.computePassInsertDebugMarker(GpuHandle(pass), label)
    }

    fun computePassSetImmediates(pass: Int, rangeOffset: Int, data: ByteArray) {
        host.computePassSetImmediates(GpuHandle(pass), rangeOffset, data)
    }

    private fun storageEntry(binding: Int, type: BufferBindingType) = BindGroupLayoutEntry(
        binding = binding,
        visibility = GpuShaderStage.COMPUTE,
        buffer = BufferBindingLayout(type = type, minBindingSize = 4),
    )
}
