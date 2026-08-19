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

    /** W3+: host-fixed 1×1 texture descriptor (not from Guest). */
    fun deviceCreateTexture(device: Int): Int = unsupported("deviceCreateTexture")

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

    /** S6+: guest `gpu-pipeline-layout-descriptor`; L2 still host-fixed empty bind-group-layouts. */
    fun deviceCreatePipelineLayout(device: Int): Int = unsupported("deviceCreatePipelineLayout")

    /** S6+: guest `gpu-bind-group-descriptor`; L2 still host-fixed empty BGL + empty entries. */
    fun deviceCreateBindGroup(device: Int): Int = unsupported("deviceCreateBindGroup")

    /** S6+: guest `gpu-render-pipeline-descriptor`; L2 still host-fixed stub shader + triangle.
     *  Also used by `[method]gpu-device.create-render-pipeline-async`. */
    fun deviceCreateRenderPipeline(device: Int): Int = unsupported("deviceCreateRenderPipeline")

    /** S6+: guest `gpu-compute-pipeline-descriptor`; L2 still host-fixed stub shader + empty layout.
     *  Also used by `[method]gpu-device.create-compute-pipeline-async`. */
    fun deviceCreateComputePipeline(device: Int): Int = unsupported("deviceCreateComputePipeline")

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

    fun queueSubmit1(queue: Int, commandBuffer: Int) {
        unsupported("queueSubmit1")
    }

    /** S6+: JNI still host-fixed 4-byte write; Guest WIT list/borrow lifted in native. */
    fun queueWriteBuffer(queue: Int, buffer: Int) {
        unsupported("queueWriteBuffer")
    }

    /** S6+: JNI still host-fixed 1×1 write; Guest WIT texel copy info lifted in native. */
    fun queueWriteTexture(queue: Int, texture: Int) {
        unsupported("queueWriteTexture")
    }

    /** W3+: texture view from host-created texture (no Guest descriptor). */
    fun textureCreateView(texture: Int): Int = unsupported("textureCreateView")

    /** L2: Guest-decoded `gpu-texture-view-descriptor` dimension + aspect (Dawn ints). */
    fun textureCreateViewDescribed(
        texture: Int,
        dimension: Int,
        aspect: Int,
    ): Int = unsupported("textureCreateViewDescribed")

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
