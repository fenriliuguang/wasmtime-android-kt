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

    fun adapterRequestDevice(
        adapter: GpuHandle,
        descriptor: DeviceDescriptor = DeviceDescriptor(),
    ): GpuHandle

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

    fun renderBundleEncoderFinish(encoder: GpuHandle, label: String? = null): GpuHandle

    fun renderBundleEncoderDraw(
        encoder: GpuHandle,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    )

    fun renderBundleEncoderDrawIndexed(
        encoder: GpuHandle,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    )

    fun renderBundleEncoderSetPipeline(encoder: GpuHandle, pipeline: GpuHandle)

    fun renderBundleEncoderSetVertexBuffer(
        encoder: GpuHandle,
        slot: Int,
        buffer: GpuHandle,
        offset: Long,
        size: Long,
    )

    fun renderBundleEncoderSetIndexBuffer(
        encoder: GpuHandle,
        buffer: GpuHandle,
        format: Int,
        offset: Long,
        size: Long,
    )

    fun renderBundleEncoderSetBindGroup(
        encoder: GpuHandle,
        index: Int,
        bindGroup: GpuHandle,
    )

    fun renderBundleEncoderDrawIndirect(encoder: GpuHandle, buffer: GpuHandle, offset: Long)

    fun renderBundleEncoderDrawIndexedIndirect(encoder: GpuHandle, buffer: GpuHandle, offset: Long)

    fun renderBundleEncoderPushDebugGroup(encoder: GpuHandle, label: String)

    fun renderBundleEncoderPopDebugGroup(encoder: GpuHandle)

    fun renderBundleEncoderInsertDebugMarker(encoder: GpuHandle, label: String)

    /** Set immediates bytes on a bundle encoder (host validates; Dawn pass-through when supported). */
    fun renderBundleEncoderSetImmediates(encoder: GpuHandle, rangeOffset: Int, data: ByteArray)

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
     * Bind a host-owned Android native window used by the next
     * [canvasContextConfigure]. Guest WIT stays `gpu-canvas-context.*`
     * (no product `surface-*`). Cpu / no window: no-op.
     */
    fun bindCanvasNativeWindow(nativeWindowHandle: Long, width: Int, height: Int) {}

    /**
     * Guest `[method]gpu-canvas-context.configure`: store device/format/usage.
     * [context] `0` allocates a new [ResourceKind.CanvasContext] handle.
     * When [bindCanvasNativeWindow] ran, Dawn creates a GPUSurface internally
     * (still not a product `surface-*` name).
     */
    fun canvasContextConfigure(context: Int, device: GpuHandle, format: Int, usage: Int): GpuHandle

    /** Guest `[method]gpu-canvas-context.unconfigure`. [context] `0` is a no-op. */
    fun canvasContextUnconfigure(context: Int)

    /**
     * Guest `[method]gpu-canvas-context.get-current-texture`.
     * [context] `0` (unconfigured fixture) allocates a 1×1 texture.
     * With a bound native window, returns the swapchain texture and the host
     * clears + presents (WIT has no `present`).
     */
    fun canvasContextGetCurrentTexture(context: Int): GpuHandle

    /**
     * Guest `[method]gpu-canvas-context.get-configuration` option discriminant.
     * [context] `0` or unconfigured → `0`; configured → `1`.
     */
    fun canvasContextHasConfiguration(context: Int): Int

    /** Stored configure device handle. Call only when [canvasContextHasConfiguration] is `1`. */
    fun canvasContextConfigurationDevice(context: Int): Int

    /** Stored configure Dawn format. Call only when [canvasContextHasConfiguration] is `1`. */
    fun canvasContextConfigurationFormat(context: Int): Int

    /** Stored configure WebGPU usage bits. Call only when [canvasContextHasConfiguration] is `1`. */
    fun canvasContextConfigurationUsage(context: Int): Int

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

    /**
     * Guest `gpu-supported-limits` scalars.
     * [device] null = adapter-only query (do not construct [GpuHandle] 0).
     */
    fun supportedLimitsMaxBindGroups(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxBindGroupsPlusVertexBuffers(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxBindingsPerBindGroup(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxBufferSize(adapter: GpuHandle, device: GpuHandle?): Long

    fun supportedLimitsMaxColorAttachmentBytesPerSample(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxColorAttachments(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxComputeInvocationsPerWorkgroup(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxComputeWorkgroupSizeX(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxComputeWorkgroupSizeY(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxComputeWorkgroupSizeZ(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxComputeWorkgroupsPerDimension(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxComputeWorkgroupStorageSize(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxDynamicStorageBuffersPerPipelineLayout(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxDynamicUniformBuffersPerPipelineLayout(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxImmediateSize(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxInterStageShaderVariables(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxSampledTexturesPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxSamplersPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxStorageBufferBindingSize(adapter: GpuHandle, device: GpuHandle?): Long

    fun supportedLimitsMaxStorageBuffersInFragmentStage(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxStorageBuffersInVertexStage(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxStorageBuffersPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxStorageTexturesInFragmentStage(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxStorageTexturesInVertexStage(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxStorageTexturesPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxTextureArrayLayers(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxTextureDimension1D(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxTextureDimension2D(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxTextureDimension3D(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxUniformBufferBindingSize(adapter: GpuHandle, device: GpuHandle?): Long

    fun supportedLimitsMaxUniformBuffersPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxVertexAttributes(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxVertexBufferArrayStride(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMaxVertexBuffers(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMinStorageBufferOffsetAlignment(adapter: GpuHandle, device: GpuHandle?): Int

    fun supportedLimitsMinUniformBufferOffsetAlignment(adapter: GpuHandle, device: GpuHandle?): Int

    fun adapterInfoSubgroupMinSize(adapter: GpuHandle): Int

    fun adapterInfoSubgroupMaxSize(adapter: GpuHandle): Int

    fun adapterInfoIsFallbackAdapter(adapter: GpuHandle): Boolean

    fun adapterInfoVendor(adapter: GpuHandle): String

    fun adapterInfoArchitecture(adapter: GpuHandle): String

    fun adapterInfoDevice(adapter: GpuHandle): String

    fun adapterInfoDescription(adapter: GpuHandle): String

    fun supportedFeaturesHas(adapter: GpuHandle, value: String): Boolean

    fun wgslLanguageFeaturesHas(value: String): Boolean

    fun gpuGetPreferredCanvasFormat(): Int

    fun gpuWgslLanguageFeatures()

    /** Owning adapter for a device (device.adapter-info L2). */
    fun deviceAdapter(device: GpuHandle): GpuHandle

    /** Validate a device handle (features / limits / adapter-info getter L2). */
    fun deviceValidate(device: GpuHandle)

    fun deviceDestroy(device: GpuHandle)

    fun deviceLostInfoReason(device: GpuHandle): Int

    fun deviceLostInfoMessage(device: GpuHandle): String

    fun gpuErrorKind(device: GpuHandle): Int

    fun gpuErrorMessage(device: GpuHandle): String

    /** Validate device before lifting uncaptured-error-event.error. */
    fun uncapturedErrorEventError(device: GpuHandle)

    /** Push an error scope (WIT filter ordinal: validation=0, out-of-memory=1, internal=2). */
    fun devicePushErrorScope(device: GpuHandle, filter: Int)

    /** Pop an error scope; returns 0 when no error was captured (empty stack included). */
    fun devicePopErrorScope(device: GpuHandle): Int

    /** Validate a queue handle (on-submitted-work-done L2). */
    fun queueValidate(queue: GpuHandle)

    /** Validate a shader-module handle (get-compilation-info L2). */
    fun shaderModuleValidate(shader: GpuHandle)

    /** WIT `gpu-compilation-message.type` ordinal (error=0, warning=1, info=2). */
    fun compilationMessageType(shader: GpuHandle): Int

    fun compilationMessageLineNum(shader: GpuHandle): Long

    fun compilationMessageLinePos(shader: GpuHandle): Long

    fun compilationMessageOffset(shader: GpuHandle): Long

    fun compilationMessageLength(shader: GpuHandle): Long

    fun compilationMessageMessage(shader: GpuHandle): String

    fun compilationInfoMessagesCount(shader: GpuHandle): Int

    fun renderPipelineGetBindGroupLayout(pipeline: GpuHandle, index: Int): GpuHandle

    fun computePipelineGetBindGroupLayout(pipeline: GpuHandle, index: Int): GpuHandle

    fun computePassPushDebugGroup(pass: GpuHandle, label: String)

    fun computePassPopDebugGroup(pass: GpuHandle)

    fun computePassInsertDebugMarker(pass: GpuHandle, label: String)

    /** Set immediates bytes on a compute pass (host validates; Dawn pass-through when supported). */
    fun computePassSetImmediates(pass: GpuHandle, rangeOffset: Int, data: ByteArray)

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

    fun renderPassSetViewport(
        pass: GpuHandle,
        x: Float,
        y: Float,
        width: Float,
        height: Float,
        minDepth: Float,
        maxDepth: Float,
    )

    fun renderPassSetScissorRect(pass: GpuHandle, x: Int, y: Int, width: Int, height: Int)

    fun renderPassSetBlendConstant(pass: GpuHandle, r: Double, g: Double, b: Double, a: Double)

    fun renderPassSetStencilReference(pass: GpuHandle, reference: Int)

    fun renderPassBeginOcclusionQuery(pass: GpuHandle, queryIndex: Int)

    fun renderPassEndOcclusionQuery(pass: GpuHandle)

    /** Execute recorded bundles on the pass (empty list validates the pass only). */
    fun renderPassExecuteBundles(pass: GpuHandle, bundles: List<GpuHandle>)

    fun renderPassPushDebugGroup(pass: GpuHandle, label: String)

    fun renderPassPopDebugGroup(pass: GpuHandle)

    fun renderPassInsertDebugMarker(pass: GpuHandle, label: String)

    /** Set immediates bytes on a render pass (host validates; Dawn pass-through when supported). */
    fun renderPassSetImmediates(pass: GpuHandle, rangeOffset: Int, data: ByteArray)

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

    fun bufferLabel(buffer: GpuHandle): String

    fun bufferSetLabel(buffer: GpuHandle, label: String)

    fun bindGroupLabel(handle: GpuHandle): String

    fun bindGroupSetLabel(handle: GpuHandle, label: String)

    fun bindGroupLayoutLabel(handle: GpuHandle): String

    fun bindGroupLayoutSetLabel(handle: GpuHandle, label: String)

    fun textureLabel(handle: GpuHandle): String

    fun textureSetLabel(handle: GpuHandle, label: String)

    fun textureViewLabel(handle: GpuHandle): String

    fun textureViewSetLabel(handle: GpuHandle, label: String)

    fun samplerLabel(handle: GpuHandle): String

    fun samplerSetLabel(handle: GpuHandle, label: String)

    fun shaderModuleLabel(handle: GpuHandle): String

    fun shaderModuleSetLabel(handle: GpuHandle, label: String)

    fun pipelineLayoutLabel(handle: GpuHandle): String

    fun pipelineLayoutSetLabel(handle: GpuHandle, label: String)

    fun querySetLabel(handle: GpuHandle): String

    fun querySetSetLabel(handle: GpuHandle, label: String)

    fun deviceLabel(handle: GpuHandle): String

    fun deviceSetLabel(handle: GpuHandle, label: String)

    fun queueLabel(handle: GpuHandle): String

    fun queueSetLabel(handle: GpuHandle, label: String)

    fun commandEncoderLabel(handle: GpuHandle): String

    fun commandEncoderSetLabel(handle: GpuHandle, label: String)

    fun commandBufferLabel(handle: GpuHandle): String

    fun commandBufferSetLabel(handle: GpuHandle, label: String)

    fun computePassEncoderLabel(handle: GpuHandle): String

    fun computePassEncoderSetLabel(handle: GpuHandle, label: String)

    fun computePipelineLabel(handle: GpuHandle): String

    fun computePipelineSetLabel(handle: GpuHandle, label: String)

    fun renderBundleEncoderLabel(handle: GpuHandle): String

    fun renderBundleEncoderSetLabel(handle: GpuHandle, label: String)

    fun renderBundleLabel(handle: GpuHandle): String

    fun renderBundleSetLabel(handle: GpuHandle, label: String)

    fun renderPassEncoderLabel(handle: GpuHandle): String

    fun renderPassEncoderSetLabel(handle: GpuHandle, label: String)

    fun renderPipelineLabel(handle: GpuHandle): String

    fun renderPipelineSetLabel(handle: GpuHandle, label: String)

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
