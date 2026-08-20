package io.github.fenriliuguang.wasi.webgpu.experimental.host

/**
 * L2 Host API — experimental Dawn host mapping for wasi:webgpu.
 *
 * Scope: compute subset + minimal Android surface/render (triangle demo).
 * No Wasm runtime / ABI / Component Model dependency on this interface.
 * Callers may be Kotlin unit tests or a thin Android demo.
 *
 * Handles are opaque u32-style ids managed by the host implementation.
 *
 * Threading (surface/render): for one host instance, surface configure /
 * getCurrentTexture / present and device submit must run on the same thread
 * (see docs/mapping/threading.md).
 */
interface WasiWebGpuHost : AutoCloseable {

    // --- Instance / Adapter / Device ---

    fun requestAdapter(options: RequestAdapterOptions = RequestAdapterOptions()): GpuHandle

    fun adapterRequestDevice(adapter: GpuHandle): GpuHandle

    fun deviceGetQueue(device: GpuHandle): GpuHandle

    // --- Resources ---

    fun deviceCreateBuffer(device: GpuHandle, descriptor: BufferDescriptor): GpuHandle

    fun deviceCreateShaderModule(device: GpuHandle, descriptor: ShaderModuleDescriptor): GpuHandle

    fun deviceCreateBindGroupLayout(
        device: GpuHandle,
        descriptor: BindGroupLayoutDescriptor,
    ): GpuHandle

    fun deviceCreateBindGroup(device: GpuHandle, descriptor: BindGroupDescriptor): GpuHandle

    fun deviceCreateTexture(device: GpuHandle, descriptor: TextureDescriptor): GpuHandle

    fun deviceCreateQuerySet(device: GpuHandle, type: Int, count: Int): GpuHandle

    /** Minimal bundle-encoder create (one color format + sample count). */
    fun deviceCreateRenderBundleEncoder(
        device: GpuHandle,
        colorFormat: Int,
        sampleCount: Int,
    ): GpuHandle

    fun deviceCreateSampler(
        device: GpuHandle,
        descriptor: SamplerDescriptor = SamplerDescriptor(),
    ): GpuHandle

    fun deviceCreatePipelineLayout(
        device: GpuHandle,
        descriptor: PipelineLayoutDescriptor,
    ): GpuHandle

    fun deviceCreateComputePipeline(
        device: GpuHandle,
        descriptor: ComputePipelineDescriptor,
    ): GpuHandle

    fun deviceCreateCommandEncoder(
        device: GpuHandle,
        descriptor: CommandEncoderDescriptor = CommandEncoderDescriptor(),
    ): GpuHandle

    // --- Surface / render (Android; Cpu host throws Unsupported) ---

    /**
     * Create a GPU surface from an Android native window handle
     * (`Util.windowFromSurface`). Desktop / Cpu host: Unsupported.
     */
    fun instanceCreateSurfaceFromAndroidNativeWindow(nativeWindowHandle: Long): GpuHandle

    /**
     * Configure swapchain from surface capabilities; returns the chosen texture format.
     */
    fun surfaceConfigure(
        surface: GpuHandle,
        device: GpuHandle,
        adapter: GpuHandle,
        width: Int,
        height: Int,
    ): Int

    fun surfaceUnconfigure(surface: GpuHandle)

    fun surfaceGetCurrentTexture(surface: GpuHandle): SurfaceTextureResult

    fun surfacePresent(surface: GpuHandle)

    /**
     * Empty vertex-buffer TriangleList; shader must export `vs_main` / `fs_main`
     * (typically `@builtin(vertex_index)`).
     * @deprecated Prefer [deviceCreateRenderPipeline] (slice E).
     */
    fun deviceCreateRenderPipelineTriangle(
        device: GpuHandle,
        shader: GpuHandle,
        format: Int,
    ): GpuHandle

    /**
     * TriangleList with explicit [vertexBuffers] layouts (CM `@location` / set-vertex-buffer path).
     * @deprecated Prefer [deviceCreateRenderPipeline] (slice E).
     */
    fun deviceCreateRenderPipelineTriangleBuffers(
        device: GpuHandle,
        shader: GpuHandle,
        format: Int,
        vertexBuffers: List<VertexBufferLayout>,
    ): GpuHandle

    fun deviceCreateRenderPipeline(
        device: GpuHandle,
        descriptor: RenderPipelineDescriptor,
    ): GpuHandle

    fun textureCreateView(
        texture: GpuHandle,
        descriptor: TextureViewDescriptor = TextureViewDescriptor(),
    ): GpuHandle

    fun textureWidth(texture: GpuHandle): Int

    fun textureHeight(texture: GpuHandle): Int

    fun textureDepthOrArrayLayers(texture: GpuHandle): Int

    fun textureMipLevelCount(texture: GpuHandle): Int

    fun textureSampleCount(texture: GpuHandle): Int

    fun textureDimension(texture: GpuHandle): Int

    fun textureFormat(texture: GpuHandle): Int

    fun textureUsage(texture: GpuHandle): Int

    /** Dawn `TextureViewDimension` int; `0` means none / unspecified. */
    fun textureBindingViewDimension(texture: GpuHandle): Int

    fun textureDestroy(texture: GpuHandle)

    fun querySetType(querySet: GpuHandle): Int

    fun querySetCount(querySet: GpuHandle): Int

    fun querySetDestroy(querySet: GpuHandle)

    fun commandEncoderResolveQuerySet(
        encoder: GpuHandle,
        querySet: GpuHandle,
        firstQuery: Int,
        queryCount: Int,
        destination: GpuHandle,
        destinationOffset: Long,
    )

    fun commandEncoderPushDebugGroup(encoder: GpuHandle, label: String)

    fun commandEncoderPopDebugGroup(encoder: GpuHandle)

    fun commandEncoderInsertDebugMarker(encoder: GpuHandle, label: String)

    /** Validate an adapter handle (features / limits / info getter L2). */
    fun adapterValidate(adapter: GpuHandle)

    /** Validate a device handle (features / limits / adapter-info getter L2). */
    fun deviceValidate(device: GpuHandle)

    fun deviceDestroy(device: GpuHandle)

    /** @deprecated Prefer [commandEncoderBeginRenderPass] (slice E). */
    fun commandEncoderBeginRenderPassClear(
        encoder: GpuHandle,
        view: GpuHandle,
        clearR: Float,
        clearG: Float,
        clearB: Float,
        clearA: Float,
    ): GpuHandle

    fun commandEncoderBeginRenderPass(
        encoder: GpuHandle,
        descriptor: RenderPassDescriptor,
    ): GpuHandle

    fun renderPassSetPipeline(pass: GpuHandle, pipeline: GpuHandle)

    fun renderPassSetBindGroup(
        pass: GpuHandle,
        index: Int,
        bindGroup: GpuHandle,
        dynamicOffsets: IntArray = intArrayOf(),
    )

    fun renderPassSetVertexBuffer(
        pass: GpuHandle,
        slot: Int,
        buffer: GpuHandle,
        offset: Long,
        size: Long,
    )

    fun renderPassSetIndexBuffer(
        pass: GpuHandle,
        buffer: GpuHandle,
        format: Int,
        offset: Long,
        size: Long,
    )

    fun renderPassDraw(
        pass: GpuHandle,
        vertexCount: Int,
        instanceCount: Int = 1,
        firstVertex: Int = 0,
        firstInstance: Int = 0,
    )

    fun renderPassDrawIndexed(
        pass: GpuHandle,
        indexCount: Int,
        instanceCount: Int = 1,
        firstIndex: Int = 0,
        baseVertex: Int = 0,
        firstInstance: Int = 0,
    )

    fun renderPassDrawIndirect(pass: GpuHandle, buffer: GpuHandle, offset: Long)

    fun renderPassDrawIndexedIndirect(pass: GpuHandle, buffer: GpuHandle, offset: Long)

    fun renderPassEnd(pass: GpuHandle)

    // --- Command encoding (compute) ---

    fun commandEncoderBeginComputePass(
        encoder: GpuHandle,
        descriptor: ComputePassDescriptor = ComputePassDescriptor(),
    ): GpuHandle

    fun computePassSetPipeline(pass: GpuHandle, pipeline: GpuHandle)

    fun computePassSetBindGroup(
        pass: GpuHandle,
        index: Int,
        bindGroup: GpuHandle,
        dynamicOffsets: IntArray = intArrayOf(),
    )

    fun computePassDispatchWorkgroups(
        pass: GpuHandle,
        workgroupCountX: Int,
        workgroupCountY: Int = 1,
        workgroupCountZ: Int = 1,
    )

    fun computePassDispatchWorkgroupsIndirect(pass: GpuHandle, buffer: GpuHandle, offset: Long)

    fun computePassEnd(pass: GpuHandle)

    fun commandEncoderCopyBufferToBuffer(
        encoder: GpuHandle,
        source: GpuHandle,
        sourceOffset: Long,
        destination: GpuHandle,
        destinationOffset: Long,
        size: Long,
    )

    fun commandEncoderClearBuffer(
        encoder: GpuHandle,
        buffer: GpuHandle,
        offset: Long,
        size: Long,
    )

    fun commandEncoderCopyBufferToTexture(
        encoder: GpuHandle,
        source: GpuHandle,
        destination: GpuHandle,
        width: Int,
        height: Int,
        depth: Int,
    )

    fun commandEncoderCopyTextureToBuffer(
        encoder: GpuHandle,
        source: GpuHandle,
        destination: GpuHandle,
        width: Int,
        height: Int,
        depth: Int,
    )

    fun commandEncoderCopyTextureToTexture(
        encoder: GpuHandle,
        source: GpuHandle,
        destination: GpuHandle,
        width: Int,
        height: Int,
        depth: Int,
    )

    fun commandEncoderFinish(encoder: GpuHandle, label: String? = null): GpuHandle

    // --- Queue / buffer IO ---

    fun queueWriteBuffer(
        queue: GpuHandle,
        buffer: GpuHandle,
        bufferOffset: Long,
        data: ByteArray,
    )

    /** Upload 2D texel data (origin 0,0; depthOrArrayLayers=1). */
    fun queueWriteTexture(
        queue: GpuHandle,
        texture: GpuHandle,
        data: ByteArray,
        width: Int,
        height: Int,
        bytesPerRow: Int,
    )

    fun queueSubmit(queue: GpuHandle, commandBuffers: List<GpuHandle>)

    fun bufferMapAsync(buffer: GpuHandle, mode: Int, offset: Long, size: Long)

    fun bufferGetMappedRange(buffer: GpuHandle, offset: Long, size: Long): ByteArray

    /** Write [data] into the mapped range at [offset] (buffer must be mapped). */
    fun bufferSetMappedRange(buffer: GpuHandle, offset: Long, data: ByteArray)

    fun bufferUnmap(buffer: GpuHandle)

    fun bufferSize(buffer: GpuHandle): Long

    fun bufferUsage(buffer: GpuHandle): Int

    fun bufferMapState(buffer: GpuHandle): Int

    fun bufferDestroy(buffer: GpuHandle)

    // --- Lifetime ---

    fun drop(handle: GpuHandle)

    /**
     * Drop [handle] if still live. Returns false when already dropped / unknown
     * (safe for paired View/Texture cleanup + future Guest destructor double-drop).
     */
    fun tryDrop(handle: GpuHandle): Boolean

    /**
     * Drop per-frame encoder / pass / command-buffer orphans.
     *
     * Swapchain View↔Texture pairs are [tryDrop]ped by AbiCm on present / next acquire
     * (`frameTextureByView`). This must **not** sweep Guest-owned Texture/TextureView
     * (e.g. cube depth + albedo) that live across frames. Call after [surfacePresent]
     * (or on frame abort).
     */
    fun releaseFrameResources()

    /**
     * Unconfigure + drop every live [ResourceKind.Surface] on this host.
     *
     * Needed after CM Guest `drop-triangle` when Surface/Device may still pin ANativeWindow.
     * Call before L2 re-attaches the same Android Surface.
     */
    fun releaseSurfaces()

    /**
     * Drop all GPU objects in the handle table but keep the Host / GPUInstance alive.
     *
     * Demo CM: stronger than [releaseSurfaces] (also drops Device/Adapter/…) so ANativeWindow
     * disconnects, without closing the CM Session (process-global linker recreate traps — D6).
     */
    fun releaseAllGpuObjects()

    override fun close()
}
