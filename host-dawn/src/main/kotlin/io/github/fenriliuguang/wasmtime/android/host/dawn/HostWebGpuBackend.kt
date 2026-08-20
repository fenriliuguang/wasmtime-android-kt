package io.github.fenriliuguang.wasmtime.android.host.dawn

import io.github.fenriliuguang.wasi.webgpu.experimental.abicm.AbiCmHostBindings
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ColorTargetState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ComputePipelineDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.FragmentState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.PipelineLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ProgrammableStage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPipelineDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.VertexState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.Extent3D
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuHandle
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureFormat
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPassColorAttachment
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPassDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.SamplerDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureViewDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.WasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.api.ExperimentalHostCallbacks
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackend

/**
 * [WebGpuBackend] wrapping a [WasiWebGpuHost]. SPI types stay in `runtime-api`;
 * this class is `:host-dawn` only.
 */
class HostWebGpuBackend(
    private val host: WasiWebGpuHost,
    override val id: String,
    private val closeHost: Boolean = true,
) : WebGpuBackend {
    override fun hostCallbacks(): ExperimentalHostCallbacks = ForwardingHostCallbacks(host)

    override fun close() {
        if (closeHost) {
            host.close()
        }
    }
}

private class ForwardingHostCallbacks(
    host: WasiWebGpuHost,
) : ExperimentalHostCallbacks {
    private val bindings = AbiCmHostBindings(host)

    override fun requestAdapter(): Int = bindings.requestAdapter()

    override fun adapterRequestDevice(adapter: Int): Int = bindings.adapterRequestDevice(adapter)

    override fun deviceGetQueue(device: Int): Int = bindings.deviceGetQueue(device)

    override fun createSurfaceFromNativeWindow(windowHandle: Long): Int =
        bindings.createSurfaceFromNativeWindow(windowHandle)

    override fun surfaceConfigure(
        surface: Int,
        device: Int,
        adapter: Int,
        width: Int,
        height: Int,
    ): Int = bindings.surfaceConfigure(surface, device, adapter, width, height)

    override fun surfaceGetCurrentTextureView(surface: Int): Int =
        bindings.surfaceGetCurrentTextureView(surface)

    override fun deviceCreateCommandEncoder(device: Int): Int =
        bindings.deviceCreateCommandEncoder(device)

    override fun deviceCreateCommandEncoderDescribed(device: Int, label: String): Int =
        bindings.deviceCreateCommandEncoder(device, label)

    override fun deviceCreateShaderModuleDescribed(device: Int, code: String): Int =
        bindings.deviceCreateShaderModule(device, code)

    override fun deviceCreateBindGroupLayoutDescribed(
        device: Int,
        binding: Int,
        visibility: Int,
        bufferType: Int,
    ): Int = bindings.deviceCreateBindGroupLayoutDescribed(device, binding, visibility, bufferType)

    override fun deviceCreateBindGroupDescribed(device: Int, layout: Int, label: String): Int =
        bindings.deviceCreateBindGroup(
            device,
            BindGroupDescriptor(
                layout = GpuHandle(layout),
                entries = emptyList(),
                label = label.ifEmpty { null },
            ),
        )

    override fun deviceCreatePipelineLayoutDescribed(
        device: Int,
        bindGroupLayouts: IntArray,
        label: String,
    ): Int =
        bindings.deviceCreatePipelineLayout(
            device,
            PipelineLayoutDescriptor(
                bindGroupLayouts = bindGroupLayouts.map { GpuHandle(it) },
                label = label.ifEmpty { null },
            ),
        )

    override fun deviceCreateComputePipelineDescribed(
        device: Int,
        shader: Int,
        entryPoint: String,
        layout: Int,
        label: String,
    ): Int {
        val module =
            if (shader != 0) {
                GpuHandle(shader)
            } else {
                GpuHandle(bindings.deviceCreateShaderModule(device, COMPUTE_STUB_WGSL))
            }
        val pipelineLayout =
            if (layout != 0) {
                GpuHandle(layout)
            } else {
                GpuHandle(
                    bindings.deviceCreatePipelineLayout(
                        device,
                        PipelineLayoutDescriptor(bindGroupLayouts = emptyList()),
                    ),
                )
            }
        return bindings.deviceCreateComputePipeline(
            device,
            ComputePipelineDescriptor(
                compute = ProgrammableStage(
                    module = module,
                    entryPoint = entryPoint.ifEmpty { "main" },
                ),
                layout = pipelineLayout,
                label = label.ifEmpty { null },
            ),
        )
    }

    override fun deviceCreateRenderPipelineDescribed(
        device: Int,
        vertexShader: Int,
        vertexEntry: String,
        fragmentShader: Int,
        fragmentEntry: String,
        format: Int,
        layout: Int,
        label: String,
    ): Int {
        val vertexModule =
            if (vertexShader != 0) {
                GpuHandle(vertexShader)
            } else {
                GpuHandle(bindings.deviceCreateShaderModule(device, COMPUTE_STUB_WGSL))
            }
        val fragmentModule =
            if (fragmentShader != 0) {
                GpuHandle(fragmentShader)
            } else {
                vertexModule
            }
        val pipelineLayout =
            if (layout != 0) {
                GpuHandle(layout)
            } else {
                GpuHandle(
                    bindings.deviceCreatePipelineLayout(
                        device,
                        PipelineLayoutDescriptor(bindGroupLayouts = emptyList()),
                    ),
                )
            }
        val targetFormat = if (format != 0) format else GpuTextureFormat.RGBA8_UNORM
        return bindings.deviceCreateRenderPipeline(
            device,
            RenderPipelineDescriptor(
                vertex = VertexState(
                    module = vertexModule,
                    entryPoint = vertexEntry.ifEmpty { "vs_main" },
                ),
                fragment = FragmentState(
                    module = fragmentModule,
                    entryPoint = fragmentEntry.ifEmpty { "fs_main" },
                    targets = listOf(ColorTargetState(format = targetFormat)),
                ),
                layout = pipelineLayout,
                label = label.ifEmpty { null },
            ),
        )
    }

    override fun beginComputePassDescribed(
        encoder: Int,
        beginningOfPassWriteIndex: Int,
        endOfPassWriteIndex: Int,
    ): Int = bindings.commandEncoderBeginComputePass(encoder)

    override fun beginRenderPassDescribed(
        encoder: Int,
        view: Int,
        loadOp: Int,
        storeOp: Int,
    ): Int =
        bindings.commandEncoderBeginRenderPass(
            encoder,
            RenderPassDescriptor(
                colorAttachments = listOf(
                    RenderPassColorAttachment(
                        view = GpuHandle(view),
                        loadOp = loadOp,
                        storeOp = storeOp,
                    ),
                ),
            ),
        )

    override fun deviceCreateBufferDescribed(device: Int, size: Long, usage: Int): Int =
        bindings.deviceCreateBuffer(device, size = size, usage = usage)

    override fun deviceCreateTextureDescribed(
        device: Int,
        width: Int,
        height: Int,
        depth: Int,
        format: Int,
        usage: Int,
    ): Int =
        bindings.deviceCreateTexture(
            device,
            TextureDescriptor(
                size = Extent3D(
                    width = width,
                    height = height,
                    depthOrArrayLayers = depth,
                ),
                format = format,
                usage = usage,
            ),
        )

    override fun bufferMapAsyncDescribed(buffer: Int, mode: Int, offset: Long, size: Long) {
        bindings.bufferMapAsync(buffer, mode, offset, size)
    }

    override fun bufferUnmapDescribed(buffer: Int) {
        bindings.bufferUnmap(buffer)
    }

    override fun bufferSizeDescribed(buffer: Int): Long = bindings.bufferSize(buffer)

    override fun bufferUsageDescribed(buffer: Int): Int = bindings.bufferUsage(buffer)

    override fun bufferMapStateDescribed(buffer: Int): Int = bindings.bufferMapState(buffer)

    override fun bufferDestroyDescribed(buffer: Int) {
        bindings.bufferDestroy(buffer)
    }

    override fun bufferGetMappedRangeDescribed(buffer: Int, offset: Long, size: Long): ByteArray =
        bindings.bufferGetMappedRange(buffer, offset, size)

    override fun bufferSetMappedRangeDescribed(buffer: Int, data: ByteArray, offset: Long) {
        bindings.bufferSetMappedRange(buffer, offset, data)
    }

    override fun deviceCreateQuerySet(device: Int): Int = bindings.deviceCreateQuerySet(device)

    override fun deviceCreateQuerySetDescribed(device: Int, type: Int, count: Int): Int =
        bindings.deviceCreateQuerySet(device, type, count)

    override fun deviceCreateRenderBundleEncoderDescribed(
        device: Int,
        colorFormat: Int,
        sampleCount: Int,
    ): Int = bindings.deviceCreateRenderBundleEncoder(device, colorFormat, sampleCount)

    override fun renderBundleEncoderFinishDescribed(encoder: Int, label: String): Int =
        bindings.renderBundleEncoderFinish(encoder, label.ifEmpty { null })

    override fun renderBundleEncoderDrawDescribed(
        encoder: Int,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        bindings.renderBundleEncoderDraw(
            encoder,
            vertexCount,
            instanceCount,
            firstVertex,
            firstInstance,
        )
    }

    override fun renderBundleEncoderDrawIndexedDescribed(
        encoder: Int,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        bindings.renderBundleEncoderDrawIndexed(
            encoder,
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        )
    }

    override fun renderBundleEncoderSetPipelineDescribed(encoder: Int, pipeline: Int) {
        bindings.renderBundleEncoderSetPipeline(encoder, pipeline)
    }

    override fun renderBundleEncoderSetVertexBufferDescribed(
        encoder: Int,
        slot: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        bindings.renderBundleEncoderSetVertexBuffer(encoder, slot, buffer, offset, size)
    }

    override fun renderBundleEncoderSetIndexBufferDescribed(
        encoder: Int,
        buffer: Int,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        bindings.renderBundleEncoderSetIndexBuffer(encoder, buffer, format, offset, size)
    }

    override fun renderBundleEncoderSetBindGroupDescribed(
        encoder: Int,
        index: Int,
        bindGroup: Int,
    ) {
        bindings.renderBundleEncoderSetBindGroup(encoder, index, bindGroup)
    }

    override fun renderBundleEncoderDrawIndirectDescribed(
        encoder: Int,
        buffer: Int,
        offset: Long,
    ) {
        bindings.renderBundleEncoderDrawIndirect(encoder, buffer, offset)
    }

    override fun renderBundleEncoderDrawIndexedIndirectDescribed(
        encoder: Int,
        buffer: Int,
        offset: Long,
    ) {
        bindings.renderBundleEncoderDrawIndexedIndirect(encoder, buffer, offset)
    }

    override fun renderBundleEncoderPushDebugGroupDescribed(encoder: Int, label: String) {
        bindings.renderBundleEncoderPushDebugGroup(encoder, label)
    }

    override fun renderBundleEncoderPopDebugGroupDescribed(encoder: Int) {
        bindings.renderBundleEncoderPopDebugGroup(encoder)
    }

    override fun renderBundleEncoderInsertDebugMarkerDescribed(encoder: Int, label: String) {
        bindings.renderBundleEncoderInsertDebugMarker(encoder, label)
    }

    override fun renderBundleEncoderSetImmediatesDescribed(
        encoder: Int,
        rangeOffset: Int,
        data: ByteArray,
        dataOffset: Long,
    ) {
        bindings.renderBundleEncoderSetImmediates(encoder, rangeOffset, data)
    }

    override fun querySetDestroyDescribed(querySet: Int) {
        bindings.querySetDestroy(querySet)
    }

    override fun querySetTypeDescribed(querySet: Int): Int = bindings.querySetType(querySet)

    override fun querySetCountDescribed(querySet: Int): Int = bindings.querySetCount(querySet)

    override fun commandEncoderResolveQuerySetDescribed(
        encoder: Int,
        querySet: Int,
        firstQuery: Int,
        queryCount: Int,
        destination: Int,
        destinationOffset: Long,
    ) {
        bindings.commandEncoderResolveQuerySet(
            encoder,
            querySet,
            firstQuery,
            queryCount,
            destination,
            destinationOffset,
        )
    }

    override fun commandEncoderPushDebugGroupDescribed(encoder: Int, label: String) {
        bindings.commandEncoderPushDebugGroup(encoder, label)
    }

    override fun commandEncoderPopDebugGroupDescribed(encoder: Int) {
        bindings.commandEncoderPopDebugGroup(encoder)
    }

    override fun commandEncoderInsertDebugMarkerDescribed(encoder: Int, label: String) {
        bindings.commandEncoderInsertDebugMarker(encoder, label)
    }

    override fun adapterFeaturesDescribed(adapter: Int) {
        bindings.adapterValidate(adapter)
    }

    override fun adapterLimitsDescribed(adapter: Int) {
        bindings.adapterValidate(adapter)
    }

    override fun supportedLimitsMaxBindGroupsDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxBindGroups(l2Adapter, device)
    }

    override fun supportedLimitsMaxBindGroupsPlusVertexBuffersDescribed(
        adapter: Int,
        device: Int,
    ): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxBindGroupsPlusVertexBuffers(l2Adapter, device)
    }

    override fun supportedLimitsMaxBindingsPerBindGroupDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxBindingsPerBindGroup(l2Adapter, device)
    }

    override fun supportedLimitsMaxBufferSizeDescribed(adapter: Int, device: Int): Long {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxBufferSize(l2Adapter, device)
    }

    override fun supportedLimitsMaxColorAttachmentBytesPerSampleDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxColorAttachmentBytesPerSample(l2Adapter, device)
    }

    override fun supportedLimitsMaxColorAttachmentsDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxColorAttachments(l2Adapter, device)
    }

    override fun supportedLimitsMaxComputeInvocationsPerWorkgroupDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxComputeInvocationsPerWorkgroup(l2Adapter, device)
    }

    override fun supportedLimitsMaxComputeWorkgroupSizeXDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxComputeWorkgroupSizeX(l2Adapter, device)
    }

    override fun supportedLimitsMaxComputeWorkgroupSizeYDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxComputeWorkgroupSizeY(l2Adapter, device)
    }

    override fun supportedLimitsMaxComputeWorkgroupSizeZDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxComputeWorkgroupSizeZ(l2Adapter, device)
    }

    override fun supportedLimitsMaxComputeWorkgroupsPerDimensionDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxComputeWorkgroupsPerDimension(l2Adapter, device)
    }

    override fun supportedLimitsMaxComputeWorkgroupStorageSizeDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxComputeWorkgroupStorageSize(l2Adapter, device)
    }

    override fun supportedLimitsMaxDynamicStorageBuffersPerPipelineLayoutDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxDynamicStorageBuffersPerPipelineLayout(l2Adapter, device)
    }

    override fun supportedLimitsMaxDynamicUniformBuffersPerPipelineLayoutDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxDynamicUniformBuffersPerPipelineLayout(l2Adapter, device)
    }

    override fun supportedLimitsMaxImmediateSizeDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxImmediateSize(l2Adapter, device)
    }

    override fun supportedLimitsMaxInterStageShaderVariablesDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxInterStageShaderVariables(l2Adapter, device)
    }

    override fun supportedLimitsMaxSampledTexturesPerShaderStageDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxSampledTexturesPerShaderStage(l2Adapter, device)
    }

    override fun supportedLimitsMaxSamplersPerShaderStageDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxSamplersPerShaderStage(l2Adapter, device)
    }

    override fun supportedLimitsMaxStorageBufferBindingSizeDescribed(adapter: Int, device: Int): Long {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxStorageBufferBindingSize(l2Adapter, device)
    }

    override fun supportedLimitsMaxStorageBuffersInFragmentStageDescribed(adapter: Int, device: Int): Int {
        val l2Adapter = if (adapter == 0 && device == 0) bindings.requestAdapter() else adapter
        return bindings.supportedLimitsMaxStorageBuffersInFragmentStage(l2Adapter, device)
    }

    override fun adapterInfoDescribed(adapter: Int) {
        bindings.adapterValidate(adapter)
    }

    override fun adapterInfoSubgroupMinSizeDescribed(adapter: Int): Int =
        bindings.adapterInfoSubgroupMinSize(adapter)

    override fun adapterInfoSubgroupMaxSizeDescribed(adapter: Int): Int =
        bindings.adapterInfoSubgroupMaxSize(adapter)

    override fun adapterInfoIsFallbackAdapterDescribed(adapter: Int): Int =
        if (bindings.adapterInfoIsFallbackAdapter(adapter)) 1 else 0

    override fun adapterInfoVendorDescribed(adapter: Int): String =
        bindings.adapterInfoVendor(adapter)

    override fun adapterInfoArchitectureDescribed(adapter: Int): String =
        bindings.adapterInfoArchitecture(adapter)

    override fun adapterInfoDeviceDescribed(adapter: Int): String =
        bindings.adapterInfoDevice(adapter)

    override fun adapterInfoDescriptionDescribed(adapter: Int): String =
        bindings.adapterInfoDescription(adapter)

    override fun supportedFeaturesHasDescribed(adapter: Int, value: String): Int =
        if (bindings.supportedFeaturesHas(adapter, value)) 1 else 0

    override fun wgslLanguageFeaturesHasDescribed(value: String): Int =
        if (bindings.wgslLanguageFeaturesHas(value)) 1 else 0

    override fun gpuGetPreferredCanvasFormatDescribed(): Int =
        bindings.gpuGetPreferredCanvasFormat()

    override fun gpuWgslLanguageFeaturesDescribed() {
        bindings.gpuWgslLanguageFeatures()
    }

    override fun deviceAdapterDescribed(device: Int): Int = bindings.deviceAdapter(device)

    override fun deviceFeaturesDescribed(device: Int) {
        bindings.deviceValidate(device)
    }

    override fun deviceLimitsDescribed(device: Int) {
        bindings.deviceValidate(device)
    }

    override fun deviceAdapterInfoDescribed(device: Int) {
        bindings.deviceValidate(device)
    }

    override fun deviceDestroyDescribed(device: Int) {
        bindings.deviceDestroy(device)
    }

    override fun deviceLostDescribed(device: Int) {
        bindings.deviceValidate(device)
    }

    override fun deviceLostInfoReasonDescribed(device: Int): Int =
        bindings.deviceLostInfoReason(device)

    override fun deviceLostInfoMessageDescribed(device: Int): String =
        bindings.deviceLostInfoMessage(device)

    override fun gpuErrorKindDescribed(device: Int): Int = bindings.gpuErrorKind(device)

    override fun gpuErrorMessageDescribed(device: Int): String =
        bindings.gpuErrorMessage(device)

    override fun devicePushErrorScopeDescribed(device: Int, filter: Int) {
        bindings.devicePushErrorScope(device, filter)
    }

    override fun devicePopErrorScopeDescribed(device: Int): Int =
        bindings.devicePopErrorScope(device)

    override fun deviceOnUncapturedErrorDescribed(device: Int) {
        bindings.deviceValidate(device)
    }

    override fun uncapturedErrorEventErrorDescribed(device: Int) {
        bindings.uncapturedErrorEventError(device)
    }

    override fun queueOnSubmittedWorkDoneDescribed(queue: Int) {
        bindings.queueValidate(queue)
    }

    override fun shaderModuleGetCompilationInfoDescribed(shader: Int) {
        bindings.shaderModuleValidate(shader)
    }

    override fun compilationMessageTypeDescribed(shader: Int): Int =
        bindings.compilationMessageType(shader)

    override fun compilationMessageLineNumDescribed(shader: Int): Long =
        bindings.compilationMessageLineNum(shader)

    override fun compilationMessageLinePosDescribed(shader: Int): Long =
        bindings.compilationMessageLinePos(shader)

    override fun compilationMessageOffsetDescribed(shader: Int): Long =
        bindings.compilationMessageOffset(shader)

    override fun compilationMessageLengthDescribed(shader: Int): Long =
        bindings.compilationMessageLength(shader)

    override fun compilationMessageMessageDescribed(shader: Int): String =
        bindings.compilationMessageMessage(shader)

    override fun compilationInfoMessagesCountDescribed(shader: Int): Int =
        bindings.compilationInfoMessagesCount(shader)

    override fun renderPipelineGetBindGroupLayoutDescribed(pipeline: Int, index: Int): Int =
        bindings.renderPipelineGetBindGroupLayout(pipeline, index)

    override fun computePipelineGetBindGroupLayoutDescribed(pipeline: Int, index: Int): Int =
        bindings.computePipelineGetBindGroupLayout(pipeline, index)

    override fun computePassPushDebugGroupDescribed(pass: Int, label: String) {
        bindings.computePassPushDebugGroup(pass, label)
    }

    override fun computePassPopDebugGroupDescribed(pass: Int) {
        bindings.computePassPopDebugGroup(pass)
    }

    override fun computePassInsertDebugMarkerDescribed(pass: Int, label: String) {
        bindings.computePassInsertDebugMarker(pass, label)
    }

    override fun computePassSetImmediatesDescribed(
        pass: Int,
        rangeOffset: Int,
        data: ByteArray,
        dataOffset: Long,
    ) {
        bindings.computePassSetImmediates(pass, rangeOffset, data)
    }

    override fun deviceCreateSamplerDescribed(
        device: Int,
        magFilter: Int,
        minFilter: Int,
        addressModeU: Int,
    ): Int =
        bindings.deviceCreateSampler(
            device,
            SamplerDescriptor(
                magFilter = magFilter,
                minFilter = minFilter,
                addressModeU = addressModeU,
            ),
        )

    override fun textureCreateViewDescribed(
        texture: Int,
        dimension: Int,
        aspect: Int,
    ): Int =
        bindings.textureCreateView(
            texture,
            TextureViewDescriptor(
                dimension = dimension,
                aspect = aspect,
            ),
        )

    override fun textureWidthDescribed(texture: Int): Int = bindings.textureWidth(texture)

    override fun textureHeightDescribed(texture: Int): Int = bindings.textureHeight(texture)

    override fun textureDepthOrArrayLayersDescribed(texture: Int): Int =
        bindings.textureDepthOrArrayLayers(texture)

    override fun textureMipLevelCountDescribed(texture: Int): Int =
        bindings.textureMipLevelCount(texture)

    override fun textureSampleCountDescribed(texture: Int): Int =
        bindings.textureSampleCount(texture)

    override fun textureDimensionDescribed(texture: Int): Int = bindings.textureDimension(texture)

    override fun textureFormatDescribed(texture: Int): Int = bindings.textureFormat(texture)

    override fun textureUsageDescribed(texture: Int): Int = bindings.textureUsage(texture)

    override fun textureBindingViewDimensionDescribed(texture: Int): Int =
        bindings.textureBindingViewDimension(texture)

    override fun textureDestroyDescribed(texture: Int) {
        bindings.textureDestroy(texture)
    }

    override fun commandEncoderCopyBufferToBufferDescribed(
        encoder: Int,
        source: Int,
        sourceOffset: Long,
        destination: Int,
        destinationOffset: Long,
        size: Long,
    ) {
        bindings.commandEncoderCopyBufferToBuffer(
            encoder,
            source,
            sourceOffset,
            destination,
            destinationOffset,
            size,
        )
    }

    override fun commandEncoderClearBufferDescribed(
        encoder: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        bindings.commandEncoderClearBuffer(encoder, buffer, offset, size)
    }

    override fun commandEncoderCopyBufferToTextureDescribed(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        bindings.commandEncoderCopyBufferToTexture(
            encoder,
            source,
            destination,
            width,
            height,
            depth,
        )
    }

    override fun commandEncoderCopyTextureToBufferDescribed(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        bindings.commandEncoderCopyTextureToBuffer(
            encoder,
            source,
            destination,
            width,
            height,
            depth,
        )
    }

    override fun commandEncoderCopyTextureToTextureDescribed(
        encoder: Int,
        source: Int,
        destination: Int,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        bindings.commandEncoderCopyTextureToTexture(
            encoder,
            source,
            destination,
            width,
            height,
            depth,
        )
    }

    override fun renderPassSetPipelineDescribed(pass: Int, pipeline: Int) {
        bindings.renderPassSetPipeline(pass, pipeline)
    }

    override fun renderPassSetBindGroupDescribed(pass: Int, index: Int, bindGroup: Int) {
        bindings.renderPassSetBindGroup(pass, index, bindGroup)
    }

    override fun renderPassSetVertexBufferDescribed(
        pass: Int,
        slot: Int,
        buffer: Int,
        offset: Long,
        size: Long,
    ) {
        bindings.renderPassSetVertexBuffer(pass, slot, buffer, offset, size)
    }

    override fun renderPassSetIndexBufferDescribed(
        pass: Int,
        buffer: Int,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        bindings.renderPassSetIndexBuffer(pass, buffer, format, offset, size)
    }

    override fun renderPassDrawDescribed(
        pass: Int,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        bindings.renderPassDraw(pass, vertexCount, instanceCount, firstVertex, firstInstance)
    }

    override fun renderPassDrawIndexedDescribed(
        pass: Int,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        bindings.renderPassDrawIndexed(
            pass,
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        )
    }

    override fun renderPassDrawIndirectDescribed(pass: Int, buffer: Int, offset: Long) {
        bindings.renderPassDrawIndirect(pass, buffer, offset)
    }

    override fun renderPassDrawIndexedIndirectDescribed(pass: Int, buffer: Int, offset: Long) {
        bindings.renderPassDrawIndexedIndirect(pass, buffer, offset)
    }

    override fun renderPassEndDescribed(pass: Int) {
        bindings.renderPassEnd(pass)
    }

    override fun computePassEndDescribed(pass: Int) {
        bindings.computePassEnd(pass)
    }

    override fun computePassSetPipelineDescribed(pass: Int, pipeline: Int) {
        bindings.computePassSetPipeline(pass, pipeline)
    }

    override fun computePassSetBindGroupDescribed(pass: Int, index: Int, bindGroup: Int) {
        bindings.computePassSetBindGroup(pass, index, bindGroup)
    }

    override fun computePassDispatchWorkgroupsDescribed(pass: Int, x: Int, y: Int, z: Int) {
        bindings.computePassDispatchWorkgroups(pass, x, y, z)
    }

    override fun computePassDispatchWorkgroupsIndirectDescribed(
        pass: Int,
        buffer: Int,
        offset: Long,
    ) {
        bindings.computePassDispatchWorkgroupsIndirect(pass, buffer, offset)
    }

    override fun commandEncoderFinish(encoder: Int): Int = bindings.commandEncoderFinish(encoder)

    override fun commandEncoderFinishDescribed(encoder: Int, label: String): Int =
        bindings.commandEncoderFinish(encoder, label)

    override fun queueSubmit1(queue: Int, commandBuffer: Int) {
        bindings.queueSubmit1(queue, commandBuffer)
    }

    override fun queueSubmitDescribed(queue: Int, commandBuffers: IntArray) {
        bindings.queueSubmit(queue, commandBuffers.toList())
    }

    override fun queueWriteBufferDescribed(
        queue: Int,
        buffer: Int,
        bufferOffset: Long,
        data: ByteArray,
    ) {
        bindings.queueWriteBuffer(queue, buffer, bufferOffset, data)
    }

    override fun queueWriteTextureDescribed(
        queue: Int,
        texture: Int,
        data: ByteArray,
        width: Int,
        height: Int,
        bytesPerRow: Int,
    ) {
        bindings.queueWriteTexture(queue, texture, data, width, height, bytesPerRow)
    }

    override fun renderPassSetStencilReferenceDescribed(pass: Int, reference: Int) {
        bindings.renderPassSetStencilReference(pass, reference)
    }

    override fun renderPassBeginOcclusionQueryDescribed(pass: Int, queryIndex: Int) {
        bindings.renderPassBeginOcclusionQuery(pass, queryIndex)
    }

    override fun renderPassEndOcclusionQueryDescribed(pass: Int) {
        bindings.renderPassEndOcclusionQuery(pass)
    }

    override fun renderPassExecuteBundlesDescribed(pass: Int, bundles: IntArray) {
        bindings.renderPassExecuteBundles(pass, bundles)
    }

    override fun renderPassPushDebugGroupDescribed(pass: Int, label: String) {
        bindings.renderPassPushDebugGroup(pass, label)
    }

    override fun renderPassPopDebugGroupDescribed(pass: Int) {
        bindings.renderPassPopDebugGroup(pass)
    }

    override fun renderPassInsertDebugMarkerDescribed(pass: Int, label: String) {
        bindings.renderPassInsertDebugMarker(pass, label)
    }

    override fun renderPassSetImmediatesDescribed(
        pass: Int,
        rangeOffset: Int,
        data: ByteArray,
        dataOffset: Long,
    ) {
        bindings.renderPassSetImmediates(pass, rangeOffset, data)
    }

    override fun renderPassSetBlendConstantDescribed(
        pass: Int,
        r: Double,
        g: Double,
        b: Double,
        a: Double,
    ) {
        bindings.renderPassSetBlendConstant(pass, r, g, b, a)
    }

    override fun renderPassSetScissorRectDescribed(
        pass: Int,
        x: Int,
        y: Int,
        width: Int,
        height: Int,
    ) {
        bindings.renderPassSetScissorRect(pass, x, y, width, height)
    }

    override fun renderPassSetViewportDescribed(
        pass: Int,
        x: Float,
        y: Float,
        width: Float,
        height: Float,
        minDepth: Float,
        maxDepth: Float,
    ) {
        bindings.renderPassSetViewport(pass, x, y, width, height, minDepth, maxDepth)
    }

    override fun surfacePresent(surface: Int) {
        bindings.surfacePresent(surface)
    }

    override fun surfaceUnconfigure(surface: Int) {
        bindings.surfaceUnconfigure(surface)
    }
}

private const val COMPUTE_STUB_WGSL = "@compute @workgroup_size(1) fn main() {}"
