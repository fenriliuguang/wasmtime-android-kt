package io.github.fenriliuguang.wasmtime.android.api

/**
 * Flat experimental CM host callbacks for L1 (u32 reps, not WIT resources).
 *
 * Product guests use WIT `[method]` names; this table is the JNI lowering.
 * Unwired `requestAdapter` is handled in native as guest `none` (not a trap).
 * Other methods default-throw so partial attachments stay explicit.
 */
interface ExperimentalHostCallbacks {
    fun requestAdapter(): Int = unsupported("requestAdapter")

    fun adapterRequestDevice(adapter: Int): Int = unsupported("adapterRequestDevice")

    fun deviceGetQueue(device: Int): Int = unsupported("deviceGetQueue")

    fun createSurfaceFromNativeWindow(windowHandle: Long): Int =
        unsupported("createSurfaceFromNativeWindow")

    fun surfaceConfigure(
        surface: Int,
        device: Int,
        adapter: Int,
        width: Int,
        height: Int,
    ): Int = unsupported("surfaceConfigure")

    fun surfaceGetCurrentTextureView(surface: Int): Int =
        unsupported("surfaceGetCurrentTextureView")

    fun deviceCreateCommandEncoder(device: Int): Int = unsupported("deviceCreateCommandEncoder")

    /** L2: Guest optional `gpu-command-encoder-descriptor` label (none → empty). */
    fun deviceCreateCommandEncoderDescribed(device: Int, label: String): Int =
        unsupported("deviceCreateCommandEncoderDescribed")

    /** W3 frozen: host-fixed buffer descriptor (size/usage not from Guest). */
    fun deviceCreateBuffer(device: Int): Int = unsupported("deviceCreateBuffer")

    /** S4: Guest-decoded `gpu-buffer-descriptor` size + WebGPU usage bits. */
    fun deviceCreateBufferDescribed(device: Int, size: Long, usage: Int): Int =
        unsupported("deviceCreateBufferDescribed")

    /** S6+: Guest-decoded `gpu-texture-descriptor` size/format/usage (Dawn format int). */
    fun deviceCreateTextureDescribed(
        device: Int,
        width: Int,
        height: Int,
        depth: Int,
        format: Int,
        usage: Int,
    ): Int = unsupported("deviceCreateTextureDescribed")

    /** W3+: JNI ignores Guest stub buffer; host creates MAP_READ buffer then map-async. */
    fun bufferMapAsync(buffer: Int) {
        unsupported("bufferMapAsync")
    }

    /** S6+: Guest `gpu-map-mode` + optional offset/size. */
    fun bufferMapAsyncDescribed(buffer: Int, mode: Int, offset: Long, size: Long) {
        unsupported("bufferMapAsyncDescribed")
    }

    /** S6+: host-fixed map-then-unmap leftover; L2 uses [bufferUnmapDescribed]. */
    fun bufferUnmap(buffer: Int) {
        unsupported("bufferUnmap")
    }

    /** L2: Guest buffer rep (0 → stub create in the wrap). */
    fun bufferUnmapDescribed(buffer: Int) {
        unsupported("bufferUnmapDescribed")
    }

    /** L2: Guest buffer handle → size. */
    fun bufferSizeDescribed(buffer: Int): Long = unsupported("bufferSizeDescribed")

    /** L2: Guest buffer handle → WebGPU/Dawn `GPUBufferUsage` bits. */
    fun bufferUsageDescribed(buffer: Int): Int = unsupported("bufferUsageDescribed")

    /** L2: Guest buffer handle → WIT `gpu-buffer-map-state` ordinal. */
    fun bufferMapStateDescribed(buffer: Int): Int = unsupported("bufferMapStateDescribed")

    /** L2: Guest buffer handle → destroy. */
    fun bufferDestroyDescribed(buffer: Int) {
        unsupported("bufferDestroyDescribed")
    }

    /** L2: Guest buffer handle + offset/size → mapped-range bytes. */
    fun bufferGetMappedRangeDescribed(buffer: Int, offset: Long, size: Long): ByteArray =
        unsupported("bufferGetMappedRangeDescribed")

    /** L2: Guest buffer handle + data + offset → write mapped range. */
    fun bufferSetMappedRangeDescribed(buffer: Int, data: ByteArray, offset: Long) {
        unsupported("bufferSetMappedRangeDescribed")
    }

    /** L2: Guest query-set handle → destroy. */
    fun querySetDestroyDescribed(querySet: Int) {
        unsupported("querySetDestroyDescribed")
    }

    /** L2: Guest query-set handle → WIT `gpu-query-type` ordinal. */
    fun querySetTypeDescribed(querySet: Int): Int = unsupported("querySetTypeDescribed")

    /** L2: Guest query-set handle → count. */
    fun querySetCountDescribed(querySet: Int): Int = unsupported("querySetCountDescribed")

    /** L2: Guest encoder + query-set + destination reps (0 → stub in the attach). */
    fun commandEncoderResolveQuerySetDescribed(
        encoder: Int,
        querySet: Int,
        firstQuery: Int,
        queryCount: Int,
        destination: Int,
        destinationOffset: Long,
    ) {
        unsupported("commandEncoderResolveQuerySetDescribed")
    }

    /** L2: Guest encoder handle + group label. */
    fun commandEncoderPushDebugGroupDescribed(encoder: Int, label: String) {
        unsupported("commandEncoderPushDebugGroupDescribed")
    }

    /** L2: Guest encoder handle → pop debug group. */
    fun commandEncoderPopDebugGroupDescribed(encoder: Int) {
        unsupported("commandEncoderPopDebugGroupDescribed")
    }

    /** L2: Guest encoder handle + marker label. */
    fun commandEncoderInsertDebugMarkerDescribed(encoder: Int, label: String) {
        unsupported("commandEncoderInsertDebugMarkerDescribed")
    }

    /** L2: Guest adapter handle → host validates before the local features lift. */
    fun adapterFeaturesDescribed(adapter: Int) {
        unsupported("adapterFeaturesDescribed")
    }

    /** L2: Guest adapter handle → host validates before the local limits lift. */
    fun adapterLimitsDescribed(adapter: Int) {
        unsupported("adapterLimitsDescribed")
    }

    /** L2: Guest adapter handle → host validates before the local adapter-info lift. */
    fun adapterInfoDescribed(adapter: Int) {
        unsupported("adapterInfoDescribed")
    }

    /** L2: Guest adapter handle → WIT `gpu-adapter-info.subgroup-min-size`. */
    fun adapterInfoSubgroupMinSizeDescribed(adapter: Int): Int =
        unsupported("adapterInfoSubgroupMinSizeDescribed")

    /** L2: Guest adapter handle → WIT `gpu-adapter-info.subgroup-max-size`. */
    fun adapterInfoSubgroupMaxSizeDescribed(adapter: Int): Int =
        unsupported("adapterInfoSubgroupMaxSizeDescribed")

    /** L2: Guest adapter handle → WIT `gpu-adapter-info.is-fallback-adapter` (0/1). */
    fun adapterInfoIsFallbackAdapterDescribed(adapter: Int): Int =
        unsupported("adapterInfoIsFallbackAdapterDescribed")

    /** L2: Guest adapter handle → WIT `gpu-adapter-info.vendor`. */
    fun adapterInfoVendorDescribed(adapter: Int): String =
        unsupported("adapterInfoVendorDescribed")

    /** L2: Guest adapter handle → WIT `gpu-adapter-info.architecture`. */
    fun adapterInfoArchitectureDescribed(adapter: Int): String =
        unsupported("adapterInfoArchitectureDescribed")

    /** L2: Guest adapter handle → WIT `gpu-adapter-info.device`. */
    fun adapterInfoDeviceDescribed(adapter: Int): String =
        unsupported("adapterInfoDeviceDescribed")

    /** L2: Guest adapter handle → WIT `gpu-adapter-info.description`. */
    fun adapterInfoDescriptionDescribed(adapter: Int): String =
        unsupported("adapterInfoDescriptionDescribed")

    /** L2: Guest device handle → owning adapter handle for adapter-info getters. */
    fun deviceAdapterDescribed(device: Int): Int = unsupported("deviceAdapterDescribed")

    /** L2: Guest device handle → host validates before the local features lift. */
    fun deviceFeaturesDescribed(device: Int) {
        unsupported("deviceFeaturesDescribed")
    }

    /** L2: Guest device handle → host validates before the local limits lift. */
    fun deviceLimitsDescribed(device: Int) {
        unsupported("deviceLimitsDescribed")
    }

    /** L2: Guest device handle → host validates before the local adapter-info lift. */
    fun deviceAdapterInfoDescribed(device: Int) {
        unsupported("deviceAdapterInfoDescribed")
    }

    /** L2: Guest device handle → destroy. */
    fun deviceDestroyDescribed(device: Int) {
        unsupported("deviceDestroyDescribed")
    }

    /** L2: Guest device handle → host validate (lost future stays local pending). */
    fun deviceLostDescribed(device: Int) {
        unsupported("deviceLostDescribed")
    }

    /** L2: Guest device handle + `gpu-error-filter` ordinal. */
    fun devicePushErrorScopeDescribed(device: Int, filter: Int) {
        unsupported("devicePushErrorScopeDescribed")
    }

    /** L2: Guest device handle → popped error ordinal (0 = none). */
    fun devicePopErrorScopeDescribed(device: Int): Int =
        unsupported("devicePopErrorScopeDescribed")

    /** L2: Guest device handle → host validate (uncaptured-error stream stays local empty). */
    fun deviceOnUncapturedErrorDescribed(device: Int) {
        unsupported("deviceOnUncapturedErrorDescribed")
    }

    /** L2: Guest queue handle → host validate (completion future stays local ready). */
    fun queueOnSubmittedWorkDoneDescribed(queue: Int) {
        unsupported("queueOnSubmittedWorkDoneDescribed")
    }

    /** L2: Guest shader-module handle → host validate (compilation-info stays local lift). */
    fun shaderModuleGetCompilationInfoDescribed(shader: Int) {
        unsupported("shaderModuleGetCompilationInfoDescribed")
    }

    /** L2: Guest render-pipeline rep (0 → stub in the attach) + group index → BGL rep. */
    fun renderPipelineGetBindGroupLayoutDescribed(pipeline: Int, index: Int): Int =
        unsupported("renderPipelineGetBindGroupLayoutDescribed")

    /** L2: Guest compute-pipeline rep (0 → stub in the attach) + group index → BGL rep. */
    fun computePipelineGetBindGroupLayoutDescribed(pipeline: Int, index: Int): Int =
        unsupported("computePipelineGetBindGroupLayoutDescribed")

    /** L2: Guest compute-pass handle + group label. */
    fun computePassPushDebugGroupDescribed(pass: Int, label: String) {
        unsupported("computePassPushDebugGroupDescribed")
    }

    /** L2: Guest compute-pass handle → pop debug group. */
    fun computePassPopDebugGroupDescribed(pass: Int) {
        unsupported("computePassPopDebugGroupDescribed")
    }

    /** L2: Guest compute-pass handle + marker label. */
    fun computePassInsertDebugMarkerDescribed(pass: Int, label: String) {
        unsupported("computePassInsertDebugMarkerDescribed")
    }

    /** L2: Guest compute-pass handle + immediates (range offset, bytes, data offset). */
    fun computePassSetImmediatesDescribed(
        pass: Int,
        rangeOffset: Int,
        data: ByteArray,
        dataOffset: Long,
    ) {
        unsupported("computePassSetImmediatesDescribed")
    }

    /** W3+: host-fixed 1×1 texture descriptor (not from Guest). */
    fun deviceCreateTexture(device: Int): Int = unsupported("deviceCreateTexture")

    /** Host-fixed occlusion query-set (count 1) for lift-only getter stubs. */
    fun deviceCreateQuerySet(device: Int): Int = unsupported("deviceCreateQuerySet")

    /** L2: Guest-decoded `gpu-query-set-descriptor` type ordinal + count. */
    fun deviceCreateQuerySetDescribed(device: Int, type: Int, count: Int): Int =
        unsupported("deviceCreateQuerySetDescribed")

    /** L2: Guest-decoded bundle-encoder descriptor (first color format Dawn int + sample count). */
    fun deviceCreateRenderBundleEncoderDescribed(
        device: Int,
        colorFormat: Int,
        sampleCount: Int,
    ): Int = unsupported("deviceCreateRenderBundleEncoderDescribed")

    /** W3+: host-fixed sampler descriptor (not from Guest). */
    fun deviceCreateSampler(device: Int): Int = unsupported("deviceCreateSampler")

    /** L2: Guest-decoded `gpu-sampler-descriptor` mag/min filter + address-mode-u (Dawn ints). */
    fun deviceCreateSamplerDescribed(
        device: Int,
        magFilter: Int,
        minFilter: Int,
        addressModeU: Int,
    ): Int = unsupported("deviceCreateSamplerDescribed")

    /** S6+: guest `gpu-shader-module-descriptor`; L2 uses [deviceCreateShaderModuleDescribed]. */
    fun deviceCreateShaderModule(device: Int): Int = unsupported("deviceCreateShaderModule")

    /** L2: Guest WGSL `code` (hints/label still unused). */
    fun deviceCreateShaderModuleDescribed(device: Int, code: String): Int =
        unsupported("deviceCreateShaderModuleDescribed")

    /** S6+: guest `gpu-bind-group-layout-descriptor`; L2 still host-fixed empty entries. */
    fun deviceCreateBindGroupLayout(device: Int): Int = unsupported("deviceCreateBindGroupLayout")

    /** L2: Guest first layout entry (binding / visibility / buffer type; bufferType -1 = none). */
    fun deviceCreateBindGroupLayoutDescribed(
        device: Int,
        binding: Int,
        visibility: Int,
        bufferType: Int,
    ): Int = unsupported("deviceCreateBindGroupLayoutDescribed")

    /** S6+: guest `gpu-pipeline-layout-descriptor`; L2 still host-fixed empty bind-group-layouts. */
    fun deviceCreatePipelineLayout(device: Int): Int = unsupported("deviceCreatePipelineLayout")

    /** L2: Guest bind-group-layout handles + optional label (none → empty). */
    fun deviceCreatePipelineLayoutDescribed(
        device: Int,
        bindGroupLayouts: IntArray,
        label: String,
    ): Int = unsupported("deviceCreatePipelineLayoutDescribed")

    /** S6+: guest `gpu-bind-group-descriptor`; L2 still host-fixed empty BGL + empty entries. */
    fun deviceCreateBindGroup(device: Int): Int = unsupported("deviceCreateBindGroup")

    /** L2: Guest layout handle + optional label (none → empty). */
    fun deviceCreateBindGroupDescribed(device: Int, layout: Int, label: String): Int =
        unsupported("deviceCreateBindGroupDescribed")

    /** S6+: guest `gpu-render-pipeline-descriptor`; L2 leftover host-fixed stub shader + triangle.
     *  Also used by `[method]gpu-device.create-render-pipeline-async`. */
    fun deviceCreateRenderPipeline(device: Int): Int = unsupported("deviceCreateRenderPipeline")

    /** L2: Guest vertex/fragment shader handles + entry-points + format (0 = RGBA8) + layout (0 = auto) + label. */
    fun deviceCreateRenderPipelineDescribed(
        device: Int,
        vertexShader: Int,
        vertexEntry: String,
        fragmentShader: Int,
        fragmentEntry: String,
        format: Int,
        layout: Int,
        label: String,
    ): Int = unsupported("deviceCreateRenderPipelineDescribed")

    /** S6+: guest `gpu-compute-pipeline-descriptor`; L2 leftover host-fixed stub shader + empty layout.
     *  Also used by `[method]gpu-device.create-compute-pipeline-async`. */
    fun deviceCreateComputePipeline(device: Int): Int = unsupported("deviceCreateComputePipeline")

    /** L2: Guest shader handle (0 = stub WGSL) + entry-point + layout handle (0 = auto/empty) + optional label. */
    fun deviceCreateComputePipelineDescribed(
        device: Int,
        shader: Int,
        entryPoint: String,
        layout: Int,
        label: String,
    ): Int = unsupported("deviceCreateComputePipelineDescribed")

    /** W3+: host-default compute-pass descriptor leftover; L2 uses [beginComputePassDescribed]. */
    fun beginComputePass(encoder: Int): Int = unsupported("beginComputePass")

    /** L2: Guest encoder + timestamp-write indices (none → 0/0). Host still uses default descriptor. */
    fun beginComputePassDescribed(
        encoder: Int,
        beginningOfPassWriteIndex: Int,
        endOfPassWriteIndex: Int,
    ): Int = unsupported("beginComputePassDescribed")

    /** Host picks a fixed clear color (smoke). */
    fun beginRenderPassClear(encoder: Int, view: Int): Int = unsupported("beginRenderPassClear")

    /** L2: Guest encoder + first color-attachment view / load-op / store-op (view 0 → stub). */
    fun beginRenderPassDescribed(encoder: Int, view: Int, loadOp: Int, storeOp: Int): Int =
        unsupported("beginRenderPassDescribed")

    fun renderPassEnd(pass: Int) {
        unsupported("renderPassEnd")
    }

    /** L2: Guest pass rep (0 → smoke rebuild in the wrap). */
    fun renderPassEndDescribed(pass: Int) {
        unsupported("renderPassEndDescribed")
    }

    /** S6+: host-fixed triangle pipeline leftover; L2 uses [renderPassSetPipelineDescribed]. */
    fun renderPassSetPipeline(pass: Int) {
        unsupported("renderPassSetPipeline")
    }

    /** L2: Guest pass + pipeline reps (0 → stub in attach). */
    fun renderPassSetPipelineDescribed(pass: Int, pipeline: Int) {
        unsupported("renderPassSetPipelineDescribed")
    }

    /** S6+: host-fixed draw(3) leftover; L2 draw uses [renderPassDrawDescribed].
     *  Indirect uses [renderPassDrawIndirectDescribed] / [renderPassDrawIndexedIndirectDescribed]. */
    fun renderPassDraw(pass: Int) {
        unsupported("renderPassDraw")
    }

    /** L2: Guest pass rep + vertex-count / option instance-count / first-vertex / first-instance. */
    fun renderPassDrawDescribed(
        pass: Int,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        unsupported("renderPassDrawDescribed")
    }

    /** L2: Guest pass rep + index-count / option instance-count / first-index / base-vertex / first-instance. */
    fun renderPassDrawIndexedDescribed(
        pass: Int,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        unsupported("renderPassDrawIndexedDescribed")
    }

    /** L2: Guest pass/buffer reps + indirect-offset. */
    fun renderPassDrawIndirectDescribed(pass: Int, buffer: Int, offset: Long) {
        unsupported("renderPassDrawIndirectDescribed")
    }

    /** L2: Guest pass/buffer reps + indirect-offset (indexed). */
    fun renderPassDrawIndexedIndirectDescribed(pass: Int, buffer: Int, offset: Long) {
        unsupported("renderPassDrawIndexedIndirectDescribed")
    }

    /** S6+: host-fixed empty bind-group leftover; L2 uses [renderPassSetBindGroupDescribed]. */
    fun renderPassSetBindGroup(pass: Int) {
        unsupported("renderPassSetBindGroup")
    }

    /** L2: Guest pass/bind-group reps + index (offsets none → empty). */
    fun renderPassSetBindGroupDescribed(pass: Int, index: Int, bindGroup: Int) {
        unsupported("renderPassSetBindGroupDescribed")
    }

    /** S6+: host-fixed VERTEX slot 0 leftover; L2 uses [renderPassSetVertexBufferDescribed]. */
    fun renderPassSetVertexBuffer(pass: Int) {
        unsupported("renderPassSetVertexBuffer")
    }

    /** L2: Guest pass/buffer reps + slot + option offset/size (none → 0). */
    fun renderPassSetVertexBufferDescribed(
        pass: Int,
        slot: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        unsupported("renderPassSetVertexBufferDescribed")
    }

    /** L2: Guest pass/buffer reps + Dawn index-format + option offset/size (none → 0). */
    fun renderPassSetIndexBufferDescribed(
        pass: Int,
        buffer: Int,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        unsupported("renderPassSetIndexBufferDescribed")
    }

    /** W3+: host-fixed begin-then-end leftover; L2 uses [computePassEndDescribed]. */
    fun computePassEnd(pass: Int) {
        unsupported("computePassEnd")
    }

    /** L2: Guest compute-pass rep (0 → smoke rebuild in the wrap). */
    fun computePassEndDescribed(pass: Int) {
        unsupported("computePassEndDescribed")
    }

    /** S6+: host-fixed compute pipeline leftover; L2 uses [computePassSetPipelineDescribed]. */
    fun computePassSetPipeline(pass: Int) {
        unsupported("computePassSetPipeline")
    }

    /** L2: Guest compute-pass + pipeline reps (0 → stub in attach). */
    fun computePassSetPipelineDescribed(pass: Int, pipeline: Int) {
        unsupported("computePassSetPipelineDescribed")
    }

    /** S6+: host-fixed empty bind-group leftover; L2 uses [computePassSetBindGroupDescribed]. */
    fun computePassSetBindGroup(pass: Int) {
        unsupported("computePassSetBindGroup")
    }

    /** L2: Guest compute-pass/bind-group reps + index (offsets none → empty). */
    fun computePassSetBindGroupDescribed(pass: Int, index: Int, bindGroup: Int) {
        unsupported("computePassSetBindGroupDescribed")
    }

    /** S6+: host-fixed 1×1×1 leftover; L2 uses [computePassDispatchWorkgroupsDescribed]. */
    fun computePassDispatchWorkgroups(pass: Int) {
        unsupported("computePassDispatchWorkgroups")
    }

    /** L2: Guest pass/buffer reps + indirect-offset. */
    fun computePassDispatchWorkgroupsIndirectDescribed(pass: Int, buffer: Int, offset: Long) {
        unsupported("computePassDispatchWorkgroupsIndirectDescribed")
    }

    /** L2: Guest pass rep + workgroup-count-x / option y/z (none → 1). */
    fun computePassDispatchWorkgroupsDescribed(pass: Int, x: Int, y: Int, z: Int) {
        unsupported("computePassDispatchWorkgroupsDescribed")
    }

    /** S6+: JNI still host-fixed 4-byte copy; Guest WIT borrow buffers are lifted in native. */
    fun commandEncoderCopyBufferToBuffer(encoder: Int) {
        unsupported("commandEncoderCopyBufferToBuffer")
    }

    /** L2: Guest encoder/buffer reps + option offsets/size (none → 0). */
    fun commandEncoderCopyBufferToBufferDescribed(
        encoder: Int,
        source: Int,
        sourceOffset: Long,
        destination: Int,
        destinationOffset: Long,
        size: Long,
    ) {
        unsupported("commandEncoderCopyBufferToBufferDescribed")
    }

    /** L2: Guest encoder/buffer reps + option offset/size (none → 0). */
    fun commandEncoderClearBufferDescribed(
        encoder: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        unsupported("commandEncoderClearBufferDescribed")
    }

    /** L2: Guest encoder/buffer/texture reps + copy-size (option height/depth → 1). */
    fun commandEncoderCopyBufferToTextureDescribed(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        unsupported("commandEncoderCopyBufferToTextureDescribed")
    }

    /** L2: Guest encoder/texture/buffer reps + copy-size (option height/depth → 1). */
    fun commandEncoderCopyTextureToBufferDescribed(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        unsupported("commandEncoderCopyTextureToBufferDescribed")
    }

    /** L2: Guest encoder/texture reps + copy-size (option height/depth → 1). */
    fun commandEncoderCopyTextureToTextureDescribed(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        unsupported("commandEncoderCopyTextureToTextureDescribed")
    }

    fun commandEncoderFinish(encoder: Int): Int = unsupported("commandEncoderFinish")

    /** L2: Guest optional `gpu-command-buffer-descriptor` label (none → empty). */
    fun commandEncoderFinishDescribed(encoder: Int, label: String): Int =
        unsupported("commandEncoderFinishDescribed")

    fun queueSubmit1(queue: Int, commandBuffer: Int) {
        unsupported("queueSubmit1")
    }

    /** L2: Guest `list<borrow<gpu-command-buffer>>` handles. */
    fun queueSubmitDescribed(queue: Int, commandBuffers: IntArray) {
        unsupported("queueSubmitDescribed")
    }

    /** S6+: JNI still host-fixed 4-byte write; Guest WIT list/borrow lifted in native. */
    fun queueWriteBuffer(queue: Int, buffer: Int) {
        unsupported("queueWriteBuffer")
    }

    /** L2: Guest buffer handle + offset + `list<u8>` payload. */
    fun queueWriteBufferDescribed(queue: Int, buffer: Int, bufferOffset: Long, data: ByteArray) {
        unsupported("queueWriteBufferDescribed")
    }

    /** S6+: JNI still host-fixed 1×1 write; Guest WIT texel copy info lifted in native. */
    fun queueWriteTexture(queue: Int, texture: Int) {
        unsupported("queueWriteTexture")
    }

    /** L2: Guest texture handle + `list<u8>` + copy width/height/bytesPerRow. */
    fun queueWriteTextureDescribed(
        queue: Int,
        texture: Int,
        data: ByteArray,
        width: Int,
        height: Int,
        bytesPerRow: Int,
    ) {
        unsupported("queueWriteTextureDescribed")
    }

    /** W3+: texture view from host-created texture (no Guest descriptor). */
    fun textureCreateView(texture: Int): Int = unsupported("textureCreateView")

    /** L2: Guest-decoded `gpu-texture-view-descriptor` dimension + aspect (Dawn ints). */
    fun textureCreateViewDescribed(
        texture: Int,
        dimension: Int,
        aspect: Int,
    ): Int = unsupported("textureCreateViewDescribed")

    /** L2: Guest texture handle → width. */
    fun textureWidthDescribed(texture: Int): Int = unsupported("textureWidthDescribed")

    /** L2: Guest texture handle → height. */
    fun textureHeightDescribed(texture: Int): Int = unsupported("textureHeightDescribed")

    /** L2: Guest texture handle → depth-or-array-layers. */
    fun textureDepthOrArrayLayersDescribed(texture: Int): Int =
        unsupported("textureDepthOrArrayLayersDescribed")

    /** L2: Guest texture handle → mip-level-count. */
    fun textureMipLevelCountDescribed(texture: Int): Int =
        unsupported("textureMipLevelCountDescribed")

    /** L2: Guest texture handle → sample-count. */
    fun textureSampleCountDescribed(texture: Int): Int =
        unsupported("textureSampleCountDescribed")

    /** L2: Guest texture handle → Dawn `TextureDimension` int. */
    fun textureDimensionDescribed(texture: Int): Int = unsupported("textureDimensionDescribed")

    /** L2: Guest texture handle → Dawn `TextureFormat` int. */
    fun textureFormatDescribed(texture: Int): Int = unsupported("textureFormatDescribed")

    /** L2: Guest texture handle → WebGPU/Dawn `GPUTextureUsage` bits. */
    fun textureUsageDescribed(texture: Int): Int = unsupported("textureUsageDescribed")

    /** L2: Guest texture handle → Dawn `TextureViewDimension` (0 = none). */
    fun textureBindingViewDimensionDescribed(texture: Int): Int =
        unsupported("textureBindingViewDimensionDescribed")

    /** L2: Guest texture handle → destroy. */
    fun textureDestroyDescribed(texture: Int) {
        unsupported("textureDestroyDescribed")
    }

    /** L2: Guest render-pass handle + viewport floats. */
    fun renderPassSetViewportDescribed(
        pass: Int,
        x: Float,
        y: Float,
        width: Float,
        height: Float,
        minDepth: Float,
        maxDepth: Float,
    ) {
        unsupported("renderPassSetViewportDescribed")
    }

    /** L2: Guest render-pass handle + scissor rect. */
    fun renderPassSetScissorRectDescribed(pass: Int, x: Int, y: Int, width: Int, height: Int) {
        unsupported("renderPassSetScissorRectDescribed")
    }

    /** L2: Guest render-pass handle + blend constant color. */
    fun renderPassSetBlendConstantDescribed(
        pass: Int,
        r: Double,
        g: Double,
        b: Double,
        a: Double,
    ) {
        unsupported("renderPassSetBlendConstantDescribed")
    }

    /** L2: Guest render-pass handle + stencil reference. */
    fun renderPassSetStencilReferenceDescribed(pass: Int, reference: Int) {
        unsupported("renderPassSetStencilReferenceDescribed")
    }

    /** L2: Guest render-pass handle + occlusion query index. */
    fun renderPassBeginOcclusionQueryDescribed(pass: Int, queryIndex: Int) {
        unsupported("renderPassBeginOcclusionQueryDescribed")
    }

    /** L2: Guest render-pass handle → end occlusion query. */
    fun renderPassEndOcclusionQueryDescribed(pass: Int) {
        unsupported("renderPassEndOcclusionQueryDescribed")
    }

    /** L2: Guest render-pass handle + bundle reps (0 entries skipped in the attach). */
    fun renderPassExecuteBundlesDescribed(pass: Int, bundles: IntArray) {
        unsupported("renderPassExecuteBundlesDescribed")
    }

    /** L2: Guest render-pass handle + group label. */
    fun renderPassPushDebugGroupDescribed(pass: Int, label: String) {
        unsupported("renderPassPushDebugGroupDescribed")
    }

    /** L2: Guest render-pass handle → pop debug group. */
    fun renderPassPopDebugGroupDescribed(pass: Int) {
        unsupported("renderPassPopDebugGroupDescribed")
    }

    /** L2: Guest render-pass handle + marker label. */
    fun renderPassInsertDebugMarkerDescribed(pass: Int, label: String) {
        unsupported("renderPassInsertDebugMarkerDescribed")
    }

    /** L2: Guest render-pass handle + immediates (range offset, bytes, data offset). */
    fun renderPassSetImmediatesDescribed(
        pass: Int,
        rangeOffset: Int,
        data: ByteArray,
        dataOffset: Long,
    ) {
        unsupported("renderPassSetImmediatesDescribed")
    }

    /** L2: Guest bundle-encoder + optional bundle label → bundle rep. */
    fun renderBundleEncoderFinishDescribed(encoder: Int, label: String): Int =
        unsupported("renderBundleEncoderFinishDescribed")

    /** L2: Guest bundle-encoder + draw counts. */
    fun renderBundleEncoderDrawDescribed(
        encoder: Int,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        unsupported("renderBundleEncoderDrawDescribed")
    }

    /** L2: Guest bundle-encoder + indexed draw counts. */
    fun renderBundleEncoderDrawIndexedDescribed(
        encoder: Int,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        unsupported("renderBundleEncoderDrawIndexedDescribed")
    }

    /** L2: Guest bundle-encoder + pipeline rep (0 → stub in the attach). */
    fun renderBundleEncoderSetPipelineDescribed(encoder: Int, pipeline: Int) {
        unsupported("renderBundleEncoderSetPipelineDescribed")
    }

    /** L2: Guest bundle-encoder + vertex-buffer slot/rep/offset/size (0 → stub in the attach). */
    fun renderBundleEncoderSetVertexBufferDescribed(
        encoder: Int,
        slot: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        unsupported("renderBundleEncoderSetVertexBufferDescribed")
    }

    /** L2: Guest bundle-encoder + index-buffer rep/format/offset/size (0 → stub in the attach). */
    fun renderBundleEncoderSetIndexBufferDescribed(
        encoder: Int,
        buffer: Int,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        unsupported("renderBundleEncoderSetIndexBufferDescribed")
    }

    /** L2: Guest bundle-encoder + bind-group index/rep (0 → stub in the attach). */
    fun renderBundleEncoderSetBindGroupDescribed(encoder: Int, index: Int, bindGroup: Int) {
        unsupported("renderBundleEncoderSetBindGroupDescribed")
    }

    /** L2: Guest bundle-encoder + indirect buffer rep/offset (0 → stub in the attach). */
    fun renderBundleEncoderDrawIndirectDescribed(encoder: Int, buffer: Int, offset: Long) {
        unsupported("renderBundleEncoderDrawIndirectDescribed")
    }

    /** L2: Guest bundle-encoder + indexed-indirect buffer rep/offset (0 → stub in the attach). */
    fun renderBundleEncoderDrawIndexedIndirectDescribed(encoder: Int, buffer: Int, offset: Long) {
        unsupported("renderBundleEncoderDrawIndexedIndirectDescribed")
    }

    /** L2: Guest bundle-encoder + group label. */
    fun renderBundleEncoderPushDebugGroupDescribed(encoder: Int, label: String) {
        unsupported("renderBundleEncoderPushDebugGroupDescribed")
    }

    /** L2: Guest bundle-encoder → pop debug group. */
    fun renderBundleEncoderPopDebugGroupDescribed(encoder: Int) {
        unsupported("renderBundleEncoderPopDebugGroupDescribed")
    }

    /** L2: Guest bundle-encoder + marker label. */
    fun renderBundleEncoderInsertDebugMarkerDescribed(encoder: Int, label: String) {
        unsupported("renderBundleEncoderInsertDebugMarkerDescribed")
    }

    /** L2: Guest bundle-encoder + immediates (range offset, bytes, data offset). */
    fun renderBundleEncoderSetImmediatesDescribed(
        encoder: Int,
        rangeOffset: Int,
        data: ByteArray,
        dataOffset: Long,
    ) {
        unsupported("renderBundleEncoderSetImmediatesDescribed")
    }

    fun surfacePresent(surface: Int) {
        unsupported("surfacePresent")
    }

    fun surfaceUnconfigure(surface: Int) {
        unsupported("surfaceUnconfigure")
    }

    private companion object {
        private fun unsupported(name: String): Nothing =
            throw UnsupportedOperationException("ExperimentalHostCallbacks.$name not wired")
    }
}
