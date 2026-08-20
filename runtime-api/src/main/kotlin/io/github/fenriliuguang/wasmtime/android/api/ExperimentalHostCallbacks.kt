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

    /** W3+: host-fixed 1×1 texture descriptor (not from Guest). */
    fun deviceCreateTexture(device: Int): Int = unsupported("deviceCreateTexture")

    /** Host-fixed occlusion query-set (count 1) for lift-only getter stubs. */
    fun deviceCreateQuerySet(device: Int): Int = unsupported("deviceCreateQuerySet")

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
