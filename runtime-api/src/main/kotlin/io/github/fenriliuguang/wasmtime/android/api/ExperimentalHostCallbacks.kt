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

    /** S6+: JNI still host-fixed map-then-unmap; Guest WIT result is lifted in native. */
    fun bufferUnmap(buffer: Int) {
        unsupported("bufferUnmap")
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

    /** S6+: guest `gpu-shader-module-descriptor`; L2 still host-fixed WGSL. */
    fun deviceCreateShaderModule(device: Int): Int = unsupported("deviceCreateShaderModule")

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

    /** W3+: host-default compute-pass descriptor (not from Guest). */
    fun beginComputePass(encoder: Int): Int = unsupported("beginComputePass")

    /** Host picks a fixed clear color (smoke). */
    fun beginRenderPassClear(encoder: Int, view: Int): Int = unsupported("beginRenderPassClear")

    fun renderPassEnd(pass: Int) {
        unsupported("renderPassEnd")
    }

    /** S6+: JNI still host-fixed; Guest WIT borrow pipeline is lifted in native. */
    fun renderPassSetPipeline(pass: Int) {
        unsupported("renderPassSetPipeline")
    }

    /** S6+: JNI still host-fixed draw(3); Guest WIT options are lifted in native.
     *  Also used by draw-indexed / draw-indirect / draw-indexed-indirect. */
    fun renderPassDraw(pass: Int) {
        unsupported("renderPassDraw")
    }

    /** S6+: JNI still host-fixed empty bind-group; Guest WIT option is lifted in native. */
    fun renderPassSetBindGroup(pass: Int) {
        unsupported("renderPassSetBindGroup")
    }

    /** S6+: JNI still host-fixed VERTEX slot 0; Guest WIT option is lifted in native.
     *  Also used by `[method]gpu-render-pass-encoder.set-index-buffer`. */
    fun renderPassSetVertexBuffer(pass: Int) {
        unsupported("renderPassSetVertexBuffer")
    }

    /** W3+: JNI ignores Guest stub pass; host begins then ends a compute pass. */
    fun computePassEnd(pass: Int) {
        unsupported("computePassEnd")
    }

    /** S6+: JNI still host-fixed; Guest WIT borrow pipeline is lifted in native. */
    fun computePassSetPipeline(pass: Int) {
        unsupported("computePassSetPipeline")
    }

    /** S6+: JNI still host-fixed empty bind-group; Guest WIT option is lifted in native. */
    fun computePassSetBindGroup(pass: Int) {
        unsupported("computePassSetBindGroup")
    }

    /** S6+: JNI still host-fixed 1×1×1; Guest WIT option counts are lifted in native.
     *  Also used by dispatch-workgroups-indirect. */
    fun computePassDispatchWorkgroups(pass: Int) {
        unsupported("computePassDispatchWorkgroups")
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

    /** L2: Guest encoder/buffer/texture reps + extent (option height/depth → 1). */
    fun commandEncoderCopyBufferToTextureDescribed(
        encoder: Int,
        buffer: Int,
        texture: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        unsupported("commandEncoderCopyBufferToTextureDescribed")
    }

    /** L2: Guest encoder/texture/buffer reps + extent (option height/depth → 1). */
    fun commandEncoderCopyTextureToBufferDescribed(
        encoder: Int,
        texture: Int,
        buffer: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        unsupported("commandEncoderCopyTextureToBufferDescribed")
    }

    /** L2: Guest encoder/src/dst texture reps + extent (option height/depth → 1). */
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

    /** L2: Guest encoder/buffer reps + option offset/size (none → 0). */
    fun commandEncoderClearBufferDescribed(
        encoder: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        unsupported("commandEncoderClearBufferDescribed")
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
