package io.github.fenriliuguang.wasmtime.android.api

/**
 * Flat experimental CM host callbacks for Track B L1 (u32 reps, not WIT resources).
 *
 * M3 uses [requestAdapter] only. M4 render smoke wires the clear→present subset.
 * Defaults throw so partial attachments stay explicit.
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

    /** W3+: host-fixed buffer descriptor (size/usage not from Guest). */
    fun deviceCreateBuffer(device: Int): Int = unsupported("deviceCreateBuffer")

    /** W3+: host-fixed 1×1 texture descriptor (not from Guest). */
    fun deviceCreateTexture(device: Int): Int = unsupported("deviceCreateTexture")

    /** W3+: host-fixed sampler descriptor (not from Guest). */
    fun deviceCreateSampler(device: Int): Int = unsupported("deviceCreateSampler")

    /** W3+: host-fixed WGSL (not from Guest). */
    fun deviceCreateShaderModule(device: Int): Int = unsupported("deviceCreateShaderModule")

    /** W3+: host-fixed empty bind-group-layout (not from Guest). */
    fun deviceCreateBindGroupLayout(device: Int): Int = unsupported("deviceCreateBindGroupLayout")

    /** W3+: host-fixed empty pipeline-layout (not from Guest). */
    fun deviceCreatePipelineLayout(device: Int): Int = unsupported("deviceCreatePipelineLayout")

    /** W3+: host-fixed empty bind-group (not from Guest). */
    fun deviceCreateBindGroup(device: Int): Int = unsupported("deviceCreateBindGroup")

    /** W3+: host-fixed stub shader + triangle pipeline (not from Guest). */
    fun deviceCreateRenderPipeline(device: Int): Int = unsupported("deviceCreateRenderPipeline")

    /** W3+: host-fixed stub shader + explicit empty pipeline-layout (not from Guest). */
    fun deviceCreateComputePipeline(device: Int): Int = unsupported("deviceCreateComputePipeline")

    /** W3+: host-default compute-pass descriptor (not from Guest). */
    fun beginComputePass(encoder: Int): Int = unsupported("beginComputePass")

    /** Host picks a fixed clear color (smoke). */
    fun beginRenderPassClear(encoder: Int, view: Int): Int = unsupported("beginRenderPassClear")

    fun renderPassEnd(pass: Int) {
        unsupported("renderPassEnd")
    }

    /** W3+: JNI ignores Guest stub pipeline; host creates triangle pipeline then set. */
    fun renderPassSetPipeline(pass: Int) {
        unsupported("renderPassSetPipeline")
    }

    /** W3+: JNI ignores Guest stub pass; host begins then ends a compute pass. */
    fun computePassEnd(pass: Int) {
        unsupported("computePassEnd")
    }

    /** W3+: JNI ignores Guest stub pipeline; host creates compute pipeline then set. */
    fun computePassSetPipeline(pass: Int) {
        unsupported("computePassSetPipeline")
    }

    /** W3+: JNI ignores Guest counts; host set-pipeline + empty bind-group then dispatch. */
    fun computePassDispatchWorkgroups(pass: Int) {
        unsupported("computePassDispatchWorkgroups")
    }

    /** W3+: JNI ignores Guest stub buffers; host creates two buffers then copy. */
    fun commandEncoderCopyBufferToBuffer(encoder: Int) {
        unsupported("commandEncoderCopyBufferToBuffer")
    }

    fun commandEncoderFinish(encoder: Int): Int = unsupported("commandEncoderFinish")

    fun queueSubmit1(queue: Int, commandBuffer: Int) {
        unsupported("queueSubmit1")
    }

    /** W3+: host-fixed 4-byte write (offset 0; buffer u32 from Guest is ignored by JNI). */
    fun queueWriteBuffer(queue: Int, buffer: Int) {
        unsupported("queueWriteBuffer")
    }

    /** W3+: host-fixed 1×1 write (texture u32 from Guest is ignored by JNI). */
    fun queueWriteTexture(queue: Int, texture: Int) {
        unsupported("queueWriteTexture")
    }

    /** W3+: texture view from host-created texture (no Guest descriptor). */
    fun textureCreateView(texture: Int): Int = unsupported("textureCreateView")

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
