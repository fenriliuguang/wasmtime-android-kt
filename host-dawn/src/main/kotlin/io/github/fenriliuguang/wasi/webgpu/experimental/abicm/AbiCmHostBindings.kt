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

    fun deviceCreateBindGroup(device: Int, descriptor: BindGroupDescriptor): Int =
        host.deviceCreateBindGroup(GpuHandle(device), descriptor).raw

    fun deviceCreateTexture(device: Int, descriptor: TextureDescriptor): Int =
        host.deviceCreateTexture(GpuHandle(device), descriptor).raw

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

    fun bufferMapAsync(buffer: Int, mode: Int, offset: Long, size: Long) {
        host.bufferMapAsync(GpuHandle(buffer), mode, offset, size)
    }

    fun bufferGetMappedRange(buffer: Int, offset: Long, size: Long): ByteArray =
        host.bufferGetMappedRange(GpuHandle(buffer), offset, size)

    fun bufferUnmap(buffer: Int) {
        host.bufferUnmap(GpuHandle(buffer))
    }

    private fun storageEntry(binding: Int, type: BufferBindingType) = BindGroupLayoutEntry(
        binding = binding,
        visibility = GpuShaderStage.COMPUTE,
        buffer = BufferBindingLayout(type = type, minBindingSize = 4),
    )
}
