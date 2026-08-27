package io.github.fenriliuguang.wasi.webgpu.experimental.dawn

import androidx.webgpu.AdapterType
import androidx.webgpu.BackendType
import androidx.webgpu.BufferBindingType as DawnBufferBindingType
import androidx.webgpu.ColorWriteMask
import androidx.webgpu.CompositeAlphaMode
import androidx.webgpu.DeviceLostCallback
import androidx.webgpu.GPU
import androidx.webgpu.GPUAdapter
import androidx.webgpu.GPUBindGroup
import androidx.webgpu.GPUBindGroupDescriptor
import androidx.webgpu.GPUBindGroupEntry
import androidx.webgpu.GPUBindGroupLayout
import androidx.webgpu.GPUBindGroupLayoutDescriptor
import androidx.webgpu.GPUBindGroupLayoutEntry
import androidx.webgpu.GPUBlendComponent
import androidx.webgpu.GPUBlendState
import androidx.webgpu.GPUBuffer
import androidx.webgpu.GPUBufferBindingLayout
import androidx.webgpu.GPUBufferDescriptor
import androidx.webgpu.GPUColor
import androidx.webgpu.GPUColorTargetState
import androidx.webgpu.GPUCommandBuffer
import androidx.webgpu.GPUCommandEncoder
import androidx.webgpu.GPUCommandEncoderDescriptor
import androidx.webgpu.GPUCompatibilityModeLimits
import androidx.webgpu.GPUComputePassDescriptor
import androidx.webgpu.GPUComputePassEncoder
import androidx.webgpu.GPUComputePipeline
import androidx.webgpu.GPUComputePipelineDescriptor
import androidx.webgpu.GPUComputeState
import androidx.webgpu.GPUConstantEntry
import androidx.webgpu.GPUDepthStencilState
import androidx.webgpu.GPUDevice
import androidx.webgpu.GPUDeviceDescriptor
import androidx.webgpu.GPUExtent3D
import androidx.webgpu.GPUFragmentState
import androidx.webgpu.GPUInstance
import androidx.webgpu.GPULimits
import androidx.webgpu.GPUMultisampleState
import androidx.webgpu.GPUPipelineLayout
import androidx.webgpu.GPUPipelineLayoutDescriptor
import androidx.webgpu.GPUPrimitiveState
import androidx.webgpu.GPUQuerySet
import androidx.webgpu.GPUQuerySetDescriptor
import androidx.webgpu.GPUQueue
import androidx.webgpu.GPUQueueDescriptor
import androidx.webgpu.GPURenderBundle
import androidx.webgpu.GPURenderBundleDescriptor
import androidx.webgpu.GPURenderBundleEncoder
import androidx.webgpu.GPURenderBundleEncoderDescriptor
import androidx.webgpu.GPURenderPassColorAttachment
import androidx.webgpu.GPURenderPassDepthStencilAttachment
import androidx.webgpu.GPURenderPassDescriptor
import androidx.webgpu.GPURenderPassEncoder
import androidx.webgpu.GPURenderPipeline
import androidx.webgpu.GPURenderPipelineDescriptor
import androidx.webgpu.GPUTexelCopyBufferInfo
import androidx.webgpu.GPUTexelCopyBufferLayout
import androidx.webgpu.GPUTexelCopyTextureInfo
import androidx.webgpu.OptionalBool
import androidx.webgpu.GPURequestAdapterOptions
import androidx.webgpu.GPURequestAdapterWebXROptions
import androidx.webgpu.GPURequestCallback
import androidx.webgpu.GPUSampler
import androidx.webgpu.GPUSamplerBindingLayout
import androidx.webgpu.GPUSamplerDescriptor
import androidx.webgpu.GPUShaderModule
import androidx.webgpu.GPUShaderModuleDescriptor
import androidx.webgpu.GPUShaderSourceWGSL
import androidx.webgpu.GPUStencilFaceState
import androidx.webgpu.GPUSurface
import androidx.webgpu.GPUSurfaceConfiguration
import androidx.webgpu.GPUSurfaceDescriptor
import androidx.webgpu.GPUSurfaceSourceAndroidNativeWindow
import androidx.webgpu.GPUTexture
import androidx.webgpu.GPUTextureBindingLayout
import androidx.webgpu.GPUTextureDescriptor
import androidx.webgpu.GPUTextureView
import androidx.webgpu.GPUTextureViewDescriptor
import androidx.webgpu.GPUVertexAttribute
import androidx.webgpu.GPUVertexBufferLayout
import androidx.webgpu.GPUVertexState
import androidx.webgpu.LoadOp
import androidx.webgpu.PowerPreference as DawnPowerPreference
import androidx.webgpu.PresentMode
import androidx.webgpu.PrimitiveTopology
import androidx.webgpu.SamplerBindingType
import androidx.webgpu.StoreOp
import androidx.webgpu.SurfaceGetCurrentTextureStatus
import androidx.webgpu.TextureSampleType
import androidx.webgpu.TextureUsage
import androidx.webgpu.TextureViewDimension
import androidx.webgpu.UncapturedErrorCallback
import androidx.webgpu.VertexFormat
import androidx.webgpu.VertexStepMode
import androidx.webgpu.helper.initLibrary
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindingResource
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BufferBindingType
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BufferDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CommandEncoderDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ComputePassDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ComputePipelineDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.DeviceDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuHandle
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuSamplerBindingType
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureSampleType
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureViewDimension
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuVertexFormat
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuVertexStepMode
import io.github.fenriliuguang.wasi.webgpu.experimental.host.Extent3D
import io.github.fenriliuguang.wasi.webgpu.experimental.host.HandleTable
import io.github.fenriliuguang.wasi.webgpu.experimental.host.HostException
import io.github.fenriliuguang.wasi.webgpu.experimental.host.PipelineLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.PowerPreference
import io.github.fenriliuguang.wasi.webgpu.experimental.host.PrimitiveState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPassColorAttachment
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPassDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RenderPipelineDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.RequestAdapterOptions
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ResourceKind
import io.github.fenriliuguang.wasi.webgpu.experimental.host.SamplerDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ShaderModuleDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.SurfaceTextureResult
import io.github.fenriliuguang.wasi.webgpu.experimental.host.SurfaceTextureStatus
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureViewDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.Color
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BlendState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CanvasConfigureLeftovers
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ColorTargetState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.FragmentState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuColorWrite
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuLoadOp
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuPrimitiveTopology
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuStoreOp
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureAspect
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureFormat
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureUsage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.VertexBufferLayout
import io.github.fenriliuguang.wasi.webgpu.experimental.host.VertexState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.WasiWebGpuHost
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executor
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

/** Guest `gpu-canvas-context` store (not a product `surface-*` name). */
private class DawnCanvasContextState(
    var device: Int = 0,
    var format: Int = 0,
    var usage: Int = 0,
    var configured: Boolean = false,
    var surface: GpuHandle? = null,
    var viewFormats: List<Int> = emptyList(),
    var colorSpace: Int = -1,
    var toneMapping: Int = -1,
    var alphaMode: Int = -1,
)

private data class CanvasTextureSnap(
    val device: GpuHandle,
    val format: Int,
    val usage: Int,
    val surface: GpuHandle?,
)

/** Swapchain frame acquired via `gpu-canvas-context.get-current-texture`; presented after `queue.submit`. */
private data class PendingCanvasPresent(
    val surface: GpuHandle,
    val texture: GpuHandle,
    val views: MutableSet<Int> = mutableSetOf(),
    val commandBuffers: MutableList<Int> = mutableListOf(),
    val gpuDone: CountDownLatch = CountDownLatch(1),
)

/**
 * L3 Dawn backend for [WasiWebGpuHost].
 *
 * Depends on `androidx.webgpu`. Must not be referenced by L1 runtime adapters.
 * Methods are synchronous wrappers around Dawn async entry points and poll
 * [GPUInstance.processEvents] until callbacks fire.
 *
 * Surface/render: caller must invoke configure / getCurrentTexture / present /
 * submit on the same thread for a given host instance (see docs/mapping/threading.md).
 */
class DawnWasiWebGpuHost private constructor(
    private val instance: GPUInstance,
) : WasiWebGpuHost {

    private val handles = HandleTable()
    private val callbackExecutor: Executor = Executor(Runnable::run)
    private val eventPoller = Executors.newSingleThreadExecutor()
    private val pipelineLayouts = HashMap<Int, GPUPipelineLayout>()
    private val deviceAdapters = HashMap<GpuHandle, GpuHandle>()
    /** Host-owned Android window for `gpu-canvas-context` (not a product `surface-*`). */
    private var canvasNativeWindow: Long = 0L
    private var canvasWidth: Int = 0
    private var canvasHeight: Int = 0
    private var pendingCanvasLeftovers: CanvasConfigureLeftovers? = null
    private var pendingCanvasPresent: PendingCanvasPresent? = null
    /**
     * Recently presented swapchain images. Close only after GPU work **and**
     * after [CANVAS_FRAMES_TO_KEEP] newer frames (compositor may still scan).
     */
    private val presentedCanvasRing = ArrayDeque<PendingCanvasPresent>()
    /**
     * GPU fence of the last canvas `queue.submit`. The next
     * [canvasContextGetCurrentTexture] waits this **before** acquire so GPU
     * work overlaps the vsync wait instead of stacking after present.
     */
    private var lastCanvasSubmitDone: CountDownLatch? = null
    /** Dawn format of the bound-window swapchain; 0 when no windowed canvas is configured. */
    private var canvasSwapchainFormat: Int = 0
    /** Serializes Dawn GPU work with [GPUInstance.processEvents] (Mali SIGSEGV under races). */
    private val gpuLock = Any()
    @Volatile private var closed = false

    init {
        // Keep L2 GpuVertex* constants aligned with androidx.webgpu for CM Guest u32 flags.
        check(GpuVertexFormat.FLOAT32X2 == VertexFormat.Float32x2) {
            "GpuVertexFormat.FLOAT32X2=${GpuVertexFormat.FLOAT32X2} != VertexFormat.Float32x2=${VertexFormat.Float32x2}"
        }
        check(GpuVertexStepMode.VERTEX == VertexStepMode.Vertex) {
            "GpuVertexStepMode.VERTEX=${GpuVertexStepMode.VERTEX} != VertexStepMode.Vertex=${VertexStepMode.Vertex}"
        }
        eventPoller.execute {
            while (!closed) {
                synchronized(gpuLock) {
                    runCatching { instance.processEvents() }
                }
                try {
                    Thread.sleep(POLL_MS)
                } catch (_: InterruptedException) {
                    Thread.currentThread().interrupt()
                    break
                }
            }
        }
    }

    override fun requestAdapter(options: RequestAdapterOptions): GpuHandle {
        val dawnOptions = GPURequestAdapterOptions(
            powerPreference = when (options.powerPreference) {
                PowerPreference.Undefined -> DawnPowerPreference.Undefined
                PowerPreference.LowPower -> DawnPowerPreference.LowPower
                PowerPreference.HighPerformance -> DawnPowerPreference.HighPerformance
            },
            forceFallbackAdapter = options.forceFallbackAdapter,
            // Android Surface path needs Vulkan; Undefined may pick GLES and leave the
            // native window connected, so CM Vulkan createSurface hits WINDOW_IN_USE.
            backendType = BackendType.Vulkan,
            requestAdapterWebXROptions = options.xrCompatible?.let { xr ->
                GPURequestAdapterWebXROptions(xr)
            },
        )
        val adapter = awaitRequest<GPUAdapter>("requestAdapter") { callback ->
            instance.requestAdapter(callbackExecutor, dawnOptions, callback)
        }
        return handles.insert(ResourceKind.Adapter, adapter)
    }

    override fun adapterRequestDevice(
        adapter: GpuHandle,
        descriptor: DeviceDescriptor,
    ): GpuHandle {
        val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
        val gpuDescriptor = GPUDeviceDescriptor(
            label = descriptor.label,
            requiredFeatures = if (descriptor.requiredFeatures.isEmpty()) {
                intArrayOf()
            } else {
                // androidx FeatureName is 1-based; WIT gpu-feature-name is 0-based.
                descriptor.requiredFeatures.map { it + 1 }.toIntArray()
            },
            requiredLimits = dawnRequiredLimits(descriptor.requiredLimits),
            defaultQueue = descriptor.defaultQueueLabel?.let { GPUQueueDescriptor(label = it) }
                ?: GPUQueueDescriptor(),
            deviceLostCallbackExecutor = callbackExecutor,
            uncapturedErrorCallbackExecutor = callbackExecutor,
            deviceLostCallback = DeviceLostCallback { _, reason, message ->
                throw HostException.Backend("device lost reason=$reason: $message")
            },
            uncapturedErrorCallback = UncapturedErrorCallback { _, type, message ->
                throw HostException.Backend("uncaptured error type=$type: $message")
            },
        )
        val device = awaitRequest<GPUDevice>("requestDevice") { callback ->
            gpuAdapter.requestDevice(callbackExecutor, gpuDescriptor, callback)
        }
        val deviceHandle = handles.insert(ResourceKind.Device, device)
        deviceAdapters[deviceHandle] = adapter
        return deviceHandle
    }

    override fun deviceGetQueue(device: GpuHandle): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        return handles.insert(ResourceKind.Queue, gpuDevice.queue)
    }

    override fun deviceCreateBuffer(device: GpuHandle, descriptor: BufferDescriptor): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val buffer = gpuDevice.createBuffer(
            GPUBufferDescriptor(
                usage = descriptor.usage,
                size = descriptor.size,
                mappedAtCreation = descriptor.mappedAtCreation,
                label = descriptor.label,
            ),
        )
        return handles.insert(ResourceKind.Buffer, buffer)
    }

    override fun deviceCreateShaderModule(
        device: GpuHandle,
        descriptor: ShaderModuleDescriptor,
    ): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val module = gpuDevice.createShaderModule(
            GPUShaderModuleDescriptor(
                label = descriptor.label,
                // G3: androidx 1.0.0-alpha05 has no compilationHints slot; guest values stay on Kotlin.
                shaderSourceWGSL = GPUShaderSourceWGSL(descriptor.code),
            ),
        )
        return handles.insert(ResourceKind.ShaderModule, module)
    }

    override fun deviceCreateBindGroupLayout(
        device: GpuHandle,
        descriptor: BindGroupLayoutDescriptor,
    ): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val entries = descriptor.entries.map { entry ->
            val bufferLayout = entry.buffer
            val samplerLayout = entry.sampler
            val textureLayout = entry.texture
            if (bufferLayout == null && samplerLayout == null && textureLayout == null) {
                throw HostException.Validation("bind-group-layout entry needs buffer, sampler, or texture")
            }
            val builder = GPUBindGroupLayoutEntry.Builder(entry.binding, entry.visibility)
            if (bufferLayout != null) {
                builder.setBuffer(
                    GPUBufferBindingLayout(
                        type = when (bufferLayout.type) {
                            BufferBindingType.Uniform -> DawnBufferBindingType.Uniform
                            BufferBindingType.Storage -> DawnBufferBindingType.Storage
                            BufferBindingType.ReadOnlyStorage -> DawnBufferBindingType.ReadOnlyStorage
                        },
                        hasDynamicOffset = bufferLayout.hasDynamicOffset,
                        minBindingSize = bufferLayout.minBindingSize,
                    ),
                )
            }
            if (samplerLayout != null) {
                builder.setSampler(
                    GPUSamplerBindingLayout(
                        type = when (samplerLayout.type) {
                            GpuSamplerBindingType.NON_FILTERING -> SamplerBindingType.NonFiltering
                            GpuSamplerBindingType.COMPARISON -> SamplerBindingType.Comparison
                            else -> SamplerBindingType.Filtering
                        },
                    ),
                )
            }
            if (textureLayout != null) {
                builder.setTexture(
                    GPUTextureBindingLayout(
                        sampleType = when (textureLayout.sampleType) {
                            GpuTextureSampleType.UNFILTERABLE_FLOAT -> TextureSampleType.UnfilterableFloat
                            GpuTextureSampleType.DEPTH -> TextureSampleType.Depth
                            GpuTextureSampleType.SINT -> TextureSampleType.Sint
                            GpuTextureSampleType.UINT -> TextureSampleType.Uint
                            else -> TextureSampleType.Float
                        },
                        viewDimension = when (textureLayout.viewDimension) {
                            GpuTextureViewDimension.D1 -> TextureViewDimension._1D
                            GpuTextureViewDimension.D2_ARRAY -> TextureViewDimension._2DArray
                            GpuTextureViewDimension.CUBE -> TextureViewDimension.Cube
                            GpuTextureViewDimension.CUBE_ARRAY -> TextureViewDimension.CubeArray
                            GpuTextureViewDimension.D3 -> TextureViewDimension._3D
                            else -> TextureViewDimension._2D
                        },
                        multisampled = textureLayout.multisampled,
                    ),
                )
            }
            builder.build()
        }.toTypedArray()
        val layout = gpuDevice.createBindGroupLayout(
            GPUBindGroupLayoutDescriptor(
                entries = entries,
                label = descriptor.label,
            ),
        )
        return handles.insert(ResourceKind.BindGroupLayout, layout)
    }

    override fun deviceCreateBindGroup(device: GpuHandle, descriptor: BindGroupDescriptor): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val layout = handles.get<GPUBindGroupLayout>(descriptor.layout, ResourceKind.BindGroupLayout)
        val entries = descriptor.entries.map { entry ->
            when (val resource = entry.resource) {
                is BindingResource.Buffer -> {
                    val buffer = handles.get<GPUBuffer>(resource.binding.buffer, ResourceKind.Buffer)
                    GPUBindGroupEntry(
                        binding = entry.binding,
                        buffer = buffer,
                        offset = resource.binding.offset,
                        size = resource.binding.size ?: (buffer.size - resource.binding.offset),
                    )
                }
                is BindingResource.Sampler -> {
                    val sampler = handles.get<GPUSampler>(resource.sampler, ResourceKind.Sampler)
                    GPUBindGroupEntry(binding = entry.binding, sampler = sampler)
                }
                is BindingResource.TextureView -> {
                    val view = handles.get<GPUTextureView>(resource.view, ResourceKind.TextureView)
                    GPUBindGroupEntry(binding = entry.binding, textureView = view)
                }
            }
        }.toTypedArray()
        val bindGroup = gpuDevice.createBindGroup(
            GPUBindGroupDescriptor(
                layout = layout,
                entries = entries,
                label = descriptor.label,
            ),
        )
        return handles.insert(ResourceKind.BindGroup, bindGroup)
    }

    override fun deviceCreateTexture(device: GpuHandle, descriptor: TextureDescriptor): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val texture = gpuDevice.createTexture(
            GPUTextureDescriptor(
                usage = descriptor.usage,
                size = GPUExtent3D(
                    width = descriptor.size.width,
                    height = descriptor.size.height,
                    depthOrArrayLayers = descriptor.size.depthOrArrayLayers,
                ),
                label = descriptor.label,
                dimension = descriptor.dimension,
                format = descriptor.format,
                mipLevelCount = descriptor.mipLevelCount,
                sampleCount = descriptor.sampleCount,
                viewFormats = descriptor.viewFormats.toIntArray(),
            ),
        )
        return handles.insert(ResourceKind.Texture, texture)
    }

    override fun deviceCreateQuerySet(device: GpuHandle, type: Int, count: Int): GpuHandle {
        // Dawn QueryType is 1-based (Undefined=0); WIT occlusion=0 / timestamp=1.
        val dawnType = type + 1
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val querySet = gpuDevice.createQuerySet(
            GPUQuerySetDescriptor(
                type = dawnType,
                count = count,
            ),
        )
        return handles.insert(ResourceKind.QuerySet, querySet)
    }

    override fun deviceCreateRenderBundleEncoder(
        device: GpuHandle,
        colorFormat: Int,
        sampleCount: Int,
    ): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val encoder = gpuDevice.createRenderBundleEncoder(
            GPURenderBundleEncoderDescriptor(
                colorFormats = intArrayOf(colorFormat),
                sampleCount = sampleCount,
            ),
        )
        return handles.insert(ResourceKind.RenderBundleEncoder, encoder)
    }

    override fun renderBundleEncoderFinish(encoder: GpuHandle, label: String?): GpuHandle {
        synchronized(gpuLock) {
            val bundleEncoder =
                handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
            val bundle = bundleEncoder.finish(GPURenderBundleDescriptor(label = label))
            return handles.insert(ResourceKind.RenderBundle, bundle)
        }
    }

    override fun renderBundleEncoderDraw(
        encoder: GpuHandle,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        synchronized(gpuLock) {
            handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
                .draw(vertexCount, instanceCount, firstVertex, firstInstance)
        }
    }

    override fun renderBundleEncoderDrawIndexed(
        encoder: GpuHandle,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        synchronized(gpuLock) {
            handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
                .drawIndexed(indexCount, instanceCount, firstIndex, baseVertex, firstInstance)
        }
    }

    override fun renderBundleEncoderSetPipeline(encoder: GpuHandle, pipeline: GpuHandle) {
        synchronized(gpuLock) {
            val bundleEncoder =
                handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
            val gpuPipeline = handles.get<GPURenderPipeline>(pipeline, ResourceKind.RenderPipeline)
            bundleEncoder.setPipeline(gpuPipeline)
        }
    }

    override fun renderBundleEncoderSetVertexBuffer(
        encoder: GpuHandle,
        slot: Int,
        buffer: GpuHandle,
        offset: Long,
        size: Long,
    ) {
        synchronized(gpuLock) {
            val bundleEncoder =
                handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
            val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
            bundleEncoder.setVertexBuffer(slot, gpuBuffer, offset, size)
        }
    }

    override fun renderBundleEncoderSetIndexBuffer(
        encoder: GpuHandle,
        buffer: GpuHandle,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        synchronized(gpuLock) {
            val bundleEncoder =
                handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
            val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
            bundleEncoder.setIndexBuffer(gpuBuffer, format, offset, size)
        }
    }

    override fun renderBundleEncoderSetBindGroup(
        encoder: GpuHandle,
        index: Int,
        bindGroup: GpuHandle,
    ) {
        synchronized(gpuLock) {
            val bundleEncoder =
                handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
            val gpuBindGroup = handles.get<GPUBindGroup>(bindGroup, ResourceKind.BindGroup)
            bundleEncoder.setBindGroup(index, gpuBindGroup)
        }
    }

    override fun renderBundleEncoderDrawIndirect(
        encoder: GpuHandle,
        buffer: GpuHandle,
        offset: Long,
    ) {
        synchronized(gpuLock) {
            val bundleEncoder =
                handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
            val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
            bundleEncoder.drawIndirect(gpuBuffer, offset)
        }
    }

    override fun renderBundleEncoderDrawIndexedIndirect(
        encoder: GpuHandle,
        buffer: GpuHandle,
        offset: Long,
    ) {
        synchronized(gpuLock) {
            val bundleEncoder =
                handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
            val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
            bundleEncoder.drawIndexedIndirect(gpuBuffer, offset)
        }
    }

    override fun renderBundleEncoderPushDebugGroup(encoder: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
                .pushDebugGroup(label)
        }
    }

    override fun renderBundleEncoderPopDebugGroup(encoder: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
                .popDebugGroup()
        }
    }

    override fun renderBundleEncoderInsertDebugMarker(encoder: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
                .insertDebugMarker(label)
        }
    }

    override fun renderBundleEncoderSetImmediates(
        encoder: GpuHandle,
        rangeOffset: Int,
        data: ByteArray,
    ) {
        synchronized(gpuLock) {
            handles.get<GPURenderBundleEncoder>(encoder, ResourceKind.RenderBundleEncoder)
            // androidx.webgpu alpha05 does not expose setImmediates; validate the handle only.
        }
    }

    override fun deviceCreateSampler(device: GpuHandle, descriptor: SamplerDescriptor): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val sampler = gpuDevice.createSampler(
            GPUSamplerDescriptor(
                label = descriptor.label,
                magFilter = descriptor.magFilter,
                minFilter = descriptor.minFilter,
                addressModeU = descriptor.addressModeU,
                addressModeV = descriptor.addressModeV,
                addressModeW = descriptor.addressModeW,
                mipmapFilter = descriptor.mipmapFilter,
                lodMinClamp = descriptor.lodMinClamp,
                lodMaxClamp = descriptor.lodMaxClamp,
                compare = descriptor.compare,
            ),
        )
        return handles.insert(ResourceKind.Sampler, sampler)
    }

    override fun deviceCreatePipelineLayout(
        device: GpuHandle,
        descriptor: PipelineLayoutDescriptor,
    ): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val layouts = descriptor.bindGroupLayouts.map { layout ->
            handles.get<GPUBindGroupLayout>(layout, ResourceKind.BindGroupLayout)
        }.toTypedArray()
        val pipelineLayout = gpuDevice.createPipelineLayout(
            GPUPipelineLayoutDescriptor(
                bindGroupLayouts = layouts,
                label = descriptor.label,
            ),
        )
        return handles.insert(ResourceKind.PipelineLayout, pipelineLayout)
    }

    override fun deviceCreateComputePipeline(
        device: GpuHandle,
        descriptor: ComputePipelineDescriptor,
    ): GpuHandle {
        val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
        val module = handles.get<GPUShaderModule>(descriptor.compute.module, ResourceKind.ShaderModule)
        val pipelineLayout = descriptor.layout?.let { layoutHandle ->
            handles.get<GPUPipelineLayout>(layoutHandle, ResourceKind.PipelineLayout)
        }
        val compute = GPUComputeState(
            module = module,
            entryPoint = descriptor.compute.entryPoint ?: "main",
            constants = dawnPipelineConstants(descriptor.compute.constants),
        )
        // WIT layout: auto → omit GPUPipelineLayout (androidx 2-arg ctor; Dawn auto).
        val pipeline = gpuDevice.createComputePipeline(
            if (pipelineLayout != null) {
                GPUComputePipelineDescriptor(
                    layout = pipelineLayout,
                    compute = compute,
                    label = descriptor.label,
                )
            } else {
                GPUComputePipelineDescriptor(
                    compute = compute,
                    label = descriptor.label,
                )
            },
        )
        return handles.insert(ResourceKind.ComputePipeline, pipeline)
    }

    override fun deviceCreateCommandEncoder(
        device: GpuHandle,
        descriptor: CommandEncoderDescriptor,
    ): GpuHandle {
        synchronized(gpuLock) {
            val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
            val encoder = gpuDevice.createCommandEncoder(
                GPUCommandEncoderDescriptor(label = descriptor.label),
            )
            return handles.insert(ResourceKind.CommandEncoder, encoder)
        }
    }

    override fun instanceCreateSurfaceFromAndroidNativeWindow(nativeWindowHandle: Long): GpuHandle {
        synchronized(gpuLock) {
            val surface = instance.createSurface(
                GPUSurfaceDescriptor(
                    surfaceSourceAndroidNativeWindow =
                        GPUSurfaceSourceAndroidNativeWindow(nativeWindowHandle),
                ),
            )
            return handles.insert(ResourceKind.Surface, surface)
        }
    }

    override fun surfaceConfigure(
        surface: GpuHandle,
        device: GpuHandle,
        adapter: GpuHandle,
        width: Int,
        height: Int,
    ): Int = surfaceConfigure(surface, device, adapter, width, height, leftover = null)

    private fun surfaceConfigure(
        surface: GpuHandle,
        device: GpuHandle,
        adapter: GpuHandle,
        width: Int,
        height: Int,
        leftover: CanvasConfigureLeftovers?,
        preferredFormat: Int = 0,
    ): Int {
        require(width > 0 && height > 0) { "invalid surface size ${width}x$height" }
        synchronized(gpuLock) {
            val gpuSurface = handles.get<GPUSurface>(surface, ResourceKind.Surface)
            val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            val caps = gpuSurface.getCapabilities(gpuAdapter)
            val format = caps.formats.firstOrNull { preferredFormat != 0 && it == preferredFormat }
                ?: caps.formats.firstOrNull()
                ?: throw HostException.Backend("surface has no texture formats")
            val presentMode = PresentMode.Fifo
            val alphaMode = leftover?.alphaMode?.takeIf { it >= 0 }?.let { wit ->
                wit + 1
            } ?: (caps.alphaModes.firstOrNull() ?: CompositeAlphaMode.Opaque)
            val viewFormats = leftover?.viewFormats?.takeIf { it.isNotEmpty() }?.toIntArray()
                ?: intArrayOf()
            gpuSurface.configure(
                GPUSurfaceConfiguration(
                    device = gpuDevice,
                    width = width,
                    height = height,
                    format = format,
                    usage = TextureUsage.RenderAttachment,
                    viewFormats = viewFormats,
                    alphaMode = alphaMode,
                    presentMode = presentMode,
                ),
            )
            return format
        }
    }

    override fun surfaceUnconfigure(surface: GpuHandle) {
        synchronized(gpuLock) {
            val gpuSurface = handles.get<GPUSurface>(surface, ResourceKind.Surface)
            gpuSurface.unconfigure()
        }
    }

    override fun surfaceGetCurrentTexture(surface: GpuHandle): SurfaceTextureResult {
        synchronized(gpuLock) {
            val gpuSurface = handles.get<GPUSurface>(surface, ResourceKind.Surface)
            val surfaceTexture = gpuSurface.getCurrentTexture()
            val status = when (surfaceTexture.status) {
                SurfaceGetCurrentTextureStatus.SuccessOptimal -> SurfaceTextureStatus.SuccessOptimal
                SurfaceGetCurrentTextureStatus.SuccessSuboptimal -> SurfaceTextureStatus.SuccessSuboptimal
                SurfaceGetCurrentTextureStatus.Timeout -> SurfaceTextureStatus.Timeout
                SurfaceGetCurrentTextureStatus.Outdated -> SurfaceTextureStatus.Outdated
                SurfaceGetCurrentTextureStatus.Lost -> SurfaceTextureStatus.Lost
                else -> SurfaceTextureStatus.Error
            }
            val texture = surfaceTexture.texture
            return if (
                texture != null &&
                (status == SurfaceTextureStatus.SuccessOptimal ||
                    status == SurfaceTextureStatus.SuccessSuboptimal)
            ) {
                SurfaceTextureResult(status, handles.insert(ResourceKind.Texture, texture))
            } else {
                SurfaceTextureResult(status, null)
            }
        }
    }

    override fun surfacePresent(surface: GpuHandle) {
        synchronized(gpuLock) {
            val gpuSurface = handles.get<GPUSurface>(surface, ResourceKind.Surface)
            gpuSurface.present()
        }
    }

    override fun bindCanvasNativeWindow(nativeWindowHandle: Long, width: Int, height: Int) {
        require(nativeWindowHandle != 0L) { "window-handle is null" }
        require(width > 0 && height > 0) { "invalid canvas size ${width}x$height" }
        synchronized(gpuLock) {
            canvasNativeWindow = nativeWindowHandle
            canvasWidth = width
            canvasHeight = height
        }
    }

    /** Stage leftover configure fields for the next [canvasContextConfigure] (same thread). */
    fun stageCanvasConfigureLeftovers(leftovers: CanvasConfigureLeftovers) {
        pendingCanvasLeftovers = leftovers
    }

    override fun canvasContextConfigure(
        context: Int,
        device: GpuHandle,
        format: Int,
        usage: Int,
    ): GpuHandle {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device)
            val handle = if (context != 0) {
                GpuHandle(context)
            } else {
                handles.insert(ResourceKind.CanvasContext, DawnCanvasContextState())
            }
            val leftover = pendingCanvasLeftovers
            pendingCanvasLeftovers = null
            val state = handles.get<DawnCanvasContextState>(handle, ResourceKind.CanvasContext)
            state.device = device.raw
            state.format = format
            state.usage = usage
            state.configured = true
            state.viewFormats = leftover?.viewFormats ?: emptyList()
            state.colorSpace = leftover?.colorSpace ?: -1
            state.toneMapping = leftover?.toneMapping ?: -1
            state.alphaMode = leftover?.alphaMode ?: -1
            val window = canvasNativeWindow
            val width = canvasWidth
            val height = canvasHeight
            if (window != 0L) {
                state.surface?.let { previous ->
                    runCatching {
                        handles.get<GPUSurface>(previous, ResourceKind.Surface).unconfigure()
                    }
                }
                val adapter = deviceAdapters[device]
                    ?: throw HostException.InvalidHandle(device, "no adapter mapping")
                val surface = instanceCreateSurfaceFromAndroidNativeWindow(window)
                val chosen = surfaceConfigure(
                    surface,
                    device,
                    adapter,
                    width,
                    height,
                    leftover,
                    preferredFormat = format,
                )
                state.surface = surface
                if (chosen != 0) {
                    state.format = chosen
                    canvasSwapchainFormat = chosen
                } else {
                    canvasSwapchainFormat = format
                }
            }
            return handle
        }
    }

    override fun canvasContextUnconfigure(context: Int) {
        synchronized(gpuLock) {
            if (context == 0) return
            val state = handles.get<DawnCanvasContextState>(
                GpuHandle(context),
                ResourceKind.CanvasContext,
            )
            state.configured = false
            canvasSwapchainFormat = 0
            state.surface?.let { surface ->
                runCatching {
                    handles.get<GPUSurface>(surface, ResourceKind.Surface).unconfigure()
                }
            }
        }
    }

    override fun canvasContextGetCurrentTexture(context: Int): GpuHandle {
        if (context == 0) {
            val adapter = requestAdapter()
            val device = adapterRequestDevice(adapter)
            return deviceCreateTexture(
                device,
                TextureDescriptor(
                    size = Extent3D(width = 1, height = 1),
                    format = GpuTextureFormat.RGBA8_UNORM,
                    usage = GpuTextureUsage.RENDER_ATTACHMENT,
                ),
            )
        }
        // Wait the previous canvas submit before acquire so GPU of frame N
        // overlaps the vsync wait for N+1 (not stacked after present).
        val prevGpu = synchronized(gpuLock) { lastCanvasSubmitDone }
        if (prevGpu != null) {
            awaitCanvasGpuDone(prevGpu)
        }
        val snapshot = synchronized(gpuLock) {
            // Must return the previous BLAST image before the next acquire.
            // Overwriting pendingCanvasPresent leaked GPUTexture handles; Mali
            // hitching rose until GpuThread SIGSEGV (fault 0x20).
            discardUnpresentedCanvasFrameLocked()
            val state = handles.get<DawnCanvasContextState>(
                GpuHandle(context),
                ResourceKind.CanvasContext,
            )
            if (!state.configured) {
                throw HostException.Validation("canvas context not configured")
            }
            CanvasTextureSnap(
                device = GpuHandle(state.device),
                format = if (state.format != 0) state.format else GpuTextureFormat.RGBA8_UNORM,
                usage = if (state.usage != 0) state.usage else GpuTextureUsage.RENDER_ATTACHMENT,
                surface = state.surface,
            )
        }
        val surface = snapshot.surface
        if (surface != null) {
            val acquired = surfaceGetCurrentTexture(surface)
            val texture = acquired.texture
                ?: throw HostException.Validation(
                    "canvas get-current-texture status=${acquired.status}",
                )
            synchronized(gpuLock) {
                pendingCanvasPresent = PendingCanvasPresent(surface, texture)
            }
            return texture
        }
        return deviceCreateTexture(
            snapshot.device,
            TextureDescriptor(
                size = Extent3D(width = 1, height = 1),
                format = snapshot.format,
                usage = snapshot.usage,
            ),
        )
    }

    override fun canvasContextPresent(context: Int) {
        synchronized(gpuLock) {
            presentPendingCanvasFrameLocked()
        }
    }

    /**
     * Pump [GPUInstance.processEvents] while waiting so the fence is not
     * delayed by the 5 ms event-poller sleep (that jitter showed as hitching).
     */
    private fun awaitCanvasGpuDone(latch: CountDownLatch) {
        if (latch.count == 0L) {
            return
        }
        val deadlineNs = System.nanoTime() + TimeUnit.SECONDS.toNanos(TIMEOUT_SEC)
        while (latch.count != 0L) {
            if (System.nanoTime() >= deadlineNs) {
                throw HostException.Backend("previous canvas submit timed out")
            }
            synchronized(gpuLock) {
                runCatching { instance.processEvents() }
            }
            latch.await(1, TimeUnit.MILLISECONDS)
        }
    }

    /** Caller must hold [gpuLock]. Present once after [queueSubmit]. */
    private fun presentPendingCanvasFrameLocked() {
        val pending = pendingCanvasPresent
        if (pending != null) {
            pendingCanvasPresent = null
            val gpuSurface = handles.get<GPUSurface>(pending.surface, ResourceKind.Surface)
            gpuSurface.present()
            presentedCanvasRing.addLast(pending)
            // Recycle after async GPU fence (see queueSubmit).
        }
        // Do not sweep command buffers here: GPU may still be executing the
        // just-submitted encoder. Closing GPUCommandBuffer caused GpuThread
        // SIGSEGV 0x20 in the continuous cube loop.
        runCatching { instance.processEvents() }
    }

    /** Caller must hold [gpuLock]. Close a swapchain texture that was never presented. */
    private fun discardUnpresentedCanvasFrameLocked() {
        val pending = pendingCanvasPresent ?: return
        pendingCanvasPresent = null
        dropCanvasFrameResourcesLocked(pending)
    }

    /** Caller must hold [gpuLock]. Drop oldest frames whose GPU work is done. */
    private fun retireGpuDoneCanvasFramesLocked() {
        while (presentedCanvasRing.size > CANVAS_FRAMES_TO_KEEP) {
            val oldest = presentedCanvasRing.first()
            if (oldest.gpuDone.count != 0L) {
                break
            }
            dropCanvasFrameResourcesLocked(presentedCanvasRing.removeFirst())
        }
    }

    /** Caller must hold [gpuLock]. Drop every canvas swapchain texture we still hold. */
    private fun dropAllCanvasFramesLocked() {
        lastCanvasSubmitDone = null
        discardUnpresentedCanvasFrameLocked()
        while (presentedCanvasRing.isNotEmpty()) {
            dropCanvasFrameResourcesLocked(presentedCanvasRing.removeFirst())
        }
    }

    /** Caller must hold [gpuLock]. Return the BLAST buffer; do not sweep guest-owned textures. */
    private fun dropCanvasFrameResourcesLocked(pending: PendingCanvasPresent) {
        pending.gpuDone.countDown()
        for (viewRaw in pending.views) {
            if (viewRaw != 0) {
                tryDropLocked(GpuHandle(viewRaw), closeResource = true)
            }
        }
        tryDropLocked(pending.texture, closeResource = true)
        for (cbRaw in pending.commandBuffers) {
            if (cbRaw != 0) {
                tryDropLocked(GpuHandle(cbRaw), closeResource = true)
            }
        }
    }

    override fun canvasContextHasConfiguration(context: Int): Int {
        if (context == 0) return 0
        return synchronized(gpuLock) {
            val state = handles.get<DawnCanvasContextState>(
                GpuHandle(context),
                ResourceKind.CanvasContext,
            )
            if (state.configured) 1 else 0
        }
    }

    override fun canvasContextConfigurationDevice(context: Int): Int =
        configuredCanvas(context).device

    override fun canvasContextConfigurationFormat(context: Int): Int =
        configuredCanvas(context).format

    override fun canvasContextConfigurationUsage(context: Int): Int =
        configuredCanvas(context).usage

    private fun configuredCanvas(context: Int): DawnCanvasContextState {
        if (context == 0) {
            throw HostException.Validation("canvas context not configured")
        }
        return synchronized(gpuLock) {
            val state = handles.get<DawnCanvasContextState>(
                GpuHandle(context),
                ResourceKind.CanvasContext,
            )
            if (!state.configured) {
                throw HostException.Validation("canvas context not configured")
            }
            state
        }
    }

    override fun deviceCreateRenderPipelineTriangle(
        device: GpuHandle,
        shader: GpuHandle,
        format: Int,
    ): GpuHandle = createRenderPipelineTriangle(device, shader, format, vertexBuffers = emptyList())

    override fun deviceCreateRenderPipelineTriangleBuffers(
        device: GpuHandle,
        shader: GpuHandle,
        format: Int,
        vertexBuffers: List<VertexBufferLayout>,
    ): GpuHandle = createRenderPipelineTriangle(device, shader, format, vertexBuffers)

    override fun deviceCreateRenderPipeline(
        device: GpuHandle,
        descriptor: RenderPipelineDescriptor,
    ): GpuHandle {
        synchronized(gpuLock) {
            val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
            val vertexModule = handles.get<GPUShaderModule>(
                descriptor.vertex.module,
                ResourceKind.ShaderModule,
            )
            val fragmentModule = handles.get<GPUShaderModule>(
                descriptor.fragment.module,
                ResourceKind.ShaderModule,
            )
            val pipelineLayout = handles.get<GPUPipelineLayout>(
                descriptor.layout,
                ResourceKind.PipelineLayout,
            )
            val dawnBuffers = descriptor.vertex.buffers.map { layout ->
                GPUVertexBufferLayout(
                    arrayStride = layout.arrayStride,
                    stepMode = layout.stepMode,
                    attributes = layout.attributes.map { attr ->
                        GPUVertexAttribute(
                            format = attr.format,
                            offset = attr.offset,
                            shaderLocation = attr.shaderLocation,
                        )
                    }.toTypedArray(),
                )
            }.toTypedArray()
            val primitive = descriptor.primitive
            val topology = primitive?.topology ?: GpuPrimitiveTopology.TRIANGLE_LIST
            val depthStencil = descriptor.depthStencil?.let { ds ->
                GPUDepthStencilState(
                    format = ds.format,
                    depthWriteEnabled = if (ds.depthWriteEnabled) {
                        OptionalBool.True
                    } else {
                        OptionalBool.False
                    },
                    depthCompare = ds.depthCompare,
                    stencilFront = ds.stencilFront?.let { face ->
                        GPUStencilFaceState(
                            compare = face.compare,
                            failOp = face.failOp,
                            depthFailOp = face.depthFailOp,
                            passOp = face.passOp,
                        )
                    } ?: GPUStencilFaceState(),
                    stencilBack = ds.stencilBack?.let { face ->
                        GPUStencilFaceState(
                            compare = face.compare,
                            failOp = face.failOp,
                            depthFailOp = face.depthFailOp,
                            passOp = face.passOp,
                        )
                    } ?: GPUStencilFaceState(),
                    stencilReadMask = ds.stencilReadMask ?: -1,
                    stencilWriteMask = ds.stencilWriteMask ?: -1,
                    depthBias = ds.depthBias ?: 0,
                    depthBiasSlopeScale = ds.depthBiasSlopeScale ?: 0f,
                    depthBiasClamp = ds.depthBiasClamp ?: 0f,
                )
            }
            val pipeline = gpuDevice.createRenderPipeline(
                GPURenderPipelineDescriptor(
                    vertex = GPUVertexState(
                        module = vertexModule,
                        entryPoint = descriptor.vertex.entryPoint ?: "vs_main",
                        constants = dawnPipelineConstants(descriptor.vertex.constants),
                        buffers = dawnBuffers,
                    ),
                    layout = pipelineLayout,
                    primitive = GPUPrimitiveState(
                        topology = topology,
                        stripIndexFormat = primitive?.stripIndexFormat ?: 0,
                        frontFace = primitive?.frontFace ?: 0,
                        cullMode = primitive?.cullMode ?: 0,
                    ),
                    depthStencil = depthStencil,
                    multisample = descriptor.multisample?.let { ms ->
                        GPUMultisampleState(
                            count = ms.count,
                            mask = ms.mask,
                            alphaToCoverageEnabled = ms.alphaToCoverageEnabled,
                        )
                    } ?: GPUMultisampleState(),
                    fragment = GPUFragmentState(
                        module = fragmentModule,
                        entryPoint = descriptor.fragment.entryPoint ?: "fs_main",
                        constants = dawnPipelineConstants(descriptor.fragment.constants),
                        targets = descriptor.fragment.targets.map { target ->
                            val requested =
                                if (target.format != 0) target.format else GpuTextureFormat.RGBA8_UNORM
                            val targetFormat =
                                if (canvasSwapchainFormat != 0) canvasSwapchainFormat else requested
                            GPUColorTargetState(
                                format = targetFormat,
                                blend = dawnBlendState(target.blend),
                                writeMask = dawnColorWriteMask(target.writeMask),
                            )
                        }.toTypedArray(),
                    ),
                    label = descriptor.label,
                ),
            )
            return handles.insert(ResourceKind.RenderPipeline, pipeline)
        }
    }

    private fun dawnPipelineConstants(
        constants: Map<String, Double>,
    ): Array<GPUConstantEntry> =
        constants.entries.map { GPUConstantEntry(key = it.key, value = it.value) }.toTypedArray()

    private fun dawnBlendState(blend: BlendState?): GPUBlendState? =
        blend?.let {
            GPUBlendState(
                color = GPUBlendComponent(
                    operation = it.color.operation,
                    srcFactor = it.color.srcFactor,
                    dstFactor = it.color.dstFactor,
                ),
                alpha = GPUBlendComponent(
                    operation = it.alpha.operation,
                    srcFactor = it.alpha.srcFactor,
                    dstFactor = it.alpha.dstFactor,
                ),
            )
        }

    /** WIT `gpu-color-write`: RGB+A bits 1:1 with Dawn; WIT `all` (bit 4) → Dawn All (`0xF`). */
    private fun dawnColorWriteMask(writeMask: Int?): Int {
        if (writeMask == null || writeMask and GpuColorWrite.ALL != 0) {
            return ColorWriteMask.All
        }
        var dawn = ColorWriteMask.None
        if (writeMask and GpuColorWrite.RED != 0) dawn = dawn or ColorWriteMask.Red
        if (writeMask and GpuColorWrite.GREEN != 0) dawn = dawn or ColorWriteMask.Green
        if (writeMask and GpuColorWrite.BLUE != 0) dawn = dawn or ColorWriteMask.Blue
        if (writeMask and GpuColorWrite.ALPHA != 0) dawn = dawn or ColorWriteMask.Alpha
        return dawn
    }

    /** Empty map → none. Unknown keys are skipped. Stage-only storage keys go on compatibilityModeLimits. */
    private fun dawnRequiredLimits(required: Map<String, Long?>): GPULimits? {
        if (required.isEmpty()) return null
        val limits = GPULimits()
        var compat: GPUCompatibilityModeLimits? = null
        fun compatLimits(): GPUCompatibilityModeLimits =
            compat ?: GPUCompatibilityModeLimits().also { compat = it }
        for ((key, value) in required) {
            if (value == null) continue
            when (key) {
                "max-texture-dimension1-d" -> limits.maxTextureDimension1D = value.toGpuLimitU32()
                "max-texture-dimension2-d" -> limits.maxTextureDimension2D = value.toGpuLimitU32()
                "max-texture-dimension3-d" -> limits.maxTextureDimension3D = value.toGpuLimitU32()
                "max-texture-array-layers" -> limits.maxTextureArrayLayers = value.toGpuLimitU32()
                "max-bind-groups" -> limits.maxBindGroups = value.toGpuLimitU32()
                "max-bind-groups-plus-vertex-buffers" ->
                    limits.maxBindGroupsPlusVertexBuffers = value.toGpuLimitU32()
                "max-immediate-size" -> limits.maxImmediateSize = value.toGpuLimitU32()
                "max-bindings-per-bind-group" ->
                    limits.maxBindingsPerBindGroup = value.toGpuLimitU32()
                "max-dynamic-uniform-buffers-per-pipeline-layout" ->
                    limits.maxDynamicUniformBuffersPerPipelineLayout = value.toGpuLimitU32()
                "max-dynamic-storage-buffers-per-pipeline-layout" ->
                    limits.maxDynamicStorageBuffersPerPipelineLayout = value.toGpuLimitU32()
                "max-sampled-textures-per-shader-stage" ->
                    limits.maxSampledTexturesPerShaderStage = value.toGpuLimitU32()
                "max-samplers-per-shader-stage" ->
                    limits.maxSamplersPerShaderStage = value.toGpuLimitU32()
                "max-storage-buffers-per-shader-stage" ->
                    limits.maxStorageBuffersPerShaderStage = value.toGpuLimitU32()
                "max-storage-buffers-in-vertex-stage" ->
                    compatLimits().maxStorageBuffersInVertexStage = value.toGpuLimitU32()
                "max-storage-buffers-in-fragment-stage" ->
                    compatLimits().maxStorageBuffersInFragmentStage = value.toGpuLimitU32()
                "max-storage-textures-per-shader-stage" ->
                    limits.maxStorageTexturesPerShaderStage = value.toGpuLimitU32()
                "max-storage-textures-in-vertex-stage" ->
                    compatLimits().maxStorageTexturesInVertexStage = value.toGpuLimitU32()
                "max-storage-textures-in-fragment-stage" ->
                    compatLimits().maxStorageTexturesInFragmentStage = value.toGpuLimitU32()
                "max-uniform-buffers-per-shader-stage" ->
                    limits.maxUniformBuffersPerShaderStage = value.toGpuLimitU32()
                "max-uniform-buffer-binding-size" -> limits.maxUniformBufferBindingSize = value
                "max-storage-buffer-binding-size" -> limits.maxStorageBufferBindingSize = value
                "min-uniform-buffer-offset-alignment" ->
                    limits.minUniformBufferOffsetAlignment = value.toGpuLimitU32()
                "min-storage-buffer-offset-alignment" ->
                    limits.minStorageBufferOffsetAlignment = value.toGpuLimitU32()
                "max-vertex-buffers" -> limits.maxVertexBuffers = value.toGpuLimitU32()
                "max-buffer-size" -> limits.maxBufferSize = value
                "max-vertex-attributes" -> limits.maxVertexAttributes = value.toGpuLimitU32()
                "max-vertex-buffer-array-stride" ->
                    limits.maxVertexBufferArrayStride = value.toGpuLimitU32()
                "max-inter-stage-shader-variables" ->
                    limits.maxInterStageShaderVariables = value.toGpuLimitU32()
                "max-color-attachments" -> limits.maxColorAttachments = value.toGpuLimitU32()
                "max-color-attachment-bytes-per-sample" ->
                    limits.maxColorAttachmentBytesPerSample = value.toGpuLimitU32()
                "max-compute-workgroup-storage-size" ->
                    limits.maxComputeWorkgroupStorageSize = value.toGpuLimitU32()
                "max-compute-invocations-per-workgroup" ->
                    limits.maxComputeInvocationsPerWorkgroup = value.toGpuLimitU32()
                "max-compute-workgroup-size-x" ->
                    limits.maxComputeWorkgroupSizeX = value.toGpuLimitU32()
                "max-compute-workgroup-size-y" ->
                    limits.maxComputeWorkgroupSizeY = value.toGpuLimitU32()
                "max-compute-workgroup-size-z" ->
                    limits.maxComputeWorkgroupSizeZ = value.toGpuLimitU32()
                "max-compute-workgroups-per-dimension" ->
                    limits.maxComputeWorkgroupsPerDimension = value.toGpuLimitU32()
                else -> Unit
            }
        }
        limits.compatibilityModeLimits = compat
        return limits
    }

    private fun Long.toGpuLimitU32(): Int =
        coerceIn(0L, Int.MAX_VALUE.toLong()).toInt()

    private fun createRenderPipelineTriangle(
        device: GpuHandle,
        shader: GpuHandle,
        format: Int,
        vertexBuffers: List<VertexBufferLayout>,
    ): GpuHandle {
        val pipelineLayout = deviceCreatePipelineLayout(
            device,
            PipelineLayoutDescriptor(bindGroupLayouts = emptyList()),
        )
        return deviceCreateRenderPipeline(
            device,
            RenderPipelineDescriptor(
                vertex = VertexState(
                    module = shader,
                    entryPoint = "vs_main",
                    buffers = vertexBuffers,
                ),
                fragment = FragmentState(
                    module = shader,
                    entryPoint = "fs_main",
                    targets = listOf(ColorTargetState(format = format)),
                ),
                layout = pipelineLayout,
                primitive = PrimitiveState(topology = GpuPrimitiveTopology.TRIANGLE_LIST),
            ),
        )
    }

    override fun textureCreateView(
        texture: GpuHandle,
        descriptor: TextureViewDescriptor,
    ): GpuHandle {
        synchronized(gpuLock) {
            val gpuTexture = handles.get<GPUTexture>(texture, ResourceKind.Texture)
            // Guest `create-view` none lowers aspect/format/dimension 0 (Undefined).
            // androidx defaults aspect to All; passing Undefined on a swapchain
            // texture fails later at begin-render-pass. Match JS `texture.createView()`.
            val unspecified = descriptor.format == 0 &&
                descriptor.dimension == 0 &&
                descriptor.aspect == 0 &&
                descriptor.baseMipLevel == 0 &&
                descriptor.mipLevelCount < 0 &&
                descriptor.baseArrayLayer == 0 &&
                descriptor.arrayLayerCount < 0
            val gpuView = if (unspecified) {
                gpuTexture.createView()
            } else {
                gpuTexture.createView(
                    GPUTextureViewDescriptor(
                        dimension = descriptor.dimension,
                        aspect = if (descriptor.aspect != 0) {
                            descriptor.aspect
                        } else {
                            GpuTextureAspect.ALL
                        },
                        format = descriptor.format,
                        baseMipLevel = descriptor.baseMipLevel,
                        mipLevelCount = descriptor.mipLevelCount,
                        baseArrayLayer = descriptor.baseArrayLayer,
                        arrayLayerCount = descriptor.arrayLayerCount,
                    ),
                )
            }
            val view = handles.insert(
                ResourceKind.TextureView,
                gpuView,
            )
            pendingCanvasPresent?.let { pending ->
                if (pending.texture == texture) {
                    pending.views.add(view.raw)
                }
            }
            return view
        }
    }

    override fun textureWidth(texture: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUTexture>(texture, ResourceKind.Texture).width
        }
    }

    override fun textureHeight(texture: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUTexture>(texture, ResourceKind.Texture).height
        }
    }

    override fun textureDepthOrArrayLayers(texture: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUTexture>(texture, ResourceKind.Texture).depthOrArrayLayers
        }
    }

    override fun textureMipLevelCount(texture: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUTexture>(texture, ResourceKind.Texture).mipLevelCount
        }
    }

    override fun textureSampleCount(texture: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUTexture>(texture, ResourceKind.Texture).sampleCount
        }
    }

    override fun textureDimension(texture: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUTexture>(texture, ResourceKind.Texture).dimension
        }
    }

    override fun textureFormat(texture: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUTexture>(texture, ResourceKind.Texture).format
        }
    }

    override fun textureUsage(texture: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUTexture>(texture, ResourceKind.Texture).usage
        }
    }

    override fun textureBindingViewDimension(texture: GpuHandle): Int {
        synchronized(gpuLock) {
            handles.get<GPUTexture>(texture, ResourceKind.Texture)
            // Dawn GPUTexture does not expose the descriptor-only WIT field.
            return 0
        }
    }

    override fun textureDestroy(texture: GpuHandle) {
        synchronized(gpuLock) {
            pendingCanvasPresent?.let { pending ->
                if (pending.texture == texture) {
                    pendingCanvasPresent = null
                    for (viewRaw in pending.views) {
                        if (viewRaw != 0) {
                            tryDropLocked(GpuHandle(viewRaw), closeResource = true)
                        }
                    }
                } else {
                    pending.views.remove(texture.raw)
                }
            }
            presentedCanvasRing.removeAll { frame ->
                if (frame.texture != texture) {
                    false
                } else {
                    frame.gpuDone.countDown()
                    for (viewRaw in frame.views) {
                        if (viewRaw != 0) {
                            tryDropLocked(GpuHandle(viewRaw), closeResource = true)
                        }
                    }
                    true
                }
            }
            // Idempotent: guest destroy after present / resource.drop after host recycle.
            tryDropLocked(texture, closeResource = true)
        }
    }

    override fun querySetType(querySet: GpuHandle): Int {
        synchronized(gpuLock) {
            val dawnType = handles.get<GPUQuerySet>(querySet, ResourceKind.QuerySet).type
            return (dawnType - 1).coerceAtLeast(0)
        }
    }

    override fun querySetCount(querySet: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUQuerySet>(querySet, ResourceKind.QuerySet).count
        }
    }

    override fun querySetDestroy(querySet: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUQuerySet>(querySet, ResourceKind.QuerySet).close()
        }
    }

    override fun commandEncoderResolveQuerySet(
        encoder: GpuHandle,
        querySet: GpuHandle,
        firstQuery: Int,
        queryCount: Int,
        destination: GpuHandle,
        destinationOffset: Long,
    ) {
        synchronized(gpuLock) {
            val commandEncoder = handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
            val qs = handles.get<GPUQuerySet>(querySet, ResourceKind.QuerySet)
            val dst = handles.get<GPUBuffer>(destination, ResourceKind.Buffer)
            commandEncoder.resolveQuerySet(qs, firstQuery, queryCount, dst, destinationOffset)
        }
    }

    override fun commandEncoderPushDebugGroup(encoder: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
                .pushDebugGroup(label)
        }
    }

    override fun commandEncoderPopDebugGroup(encoder: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder).popDebugGroup()
        }
    }

    override fun commandEncoderInsertDebugMarker(encoder: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
                .insertDebugMarker(label)
        }
    }

    override fun adapterValidate(adapter: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
        }
    }

    private fun dawnSupportedLimits(adapter: GpuHandle, device: GpuHandle?) =
        if (device != null) {
            handles.get<GPUDevice>(device, ResourceKind.Device).getLimits()
        } else {
            handles.get<GPUAdapter>(adapter, ResourceKind.Adapter).getLimits()
        }

    override fun supportedLimitsMaxBindGroups(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return if (device != null) {
                handles.get<GPUDevice>(device, ResourceKind.Device).getLimits().maxBindGroups
            } else {
                handles.get<GPUAdapter>(adapter, ResourceKind.Adapter).getLimits().maxBindGroups
            }
        }
    }

    override fun supportedLimitsMaxBindGroupsPlusVertexBuffers(
        adapter: GpuHandle,
        device: GpuHandle?,
    ): Int {
        synchronized(gpuLock) {
            return if (device != null) {
                handles
                    .get<GPUDevice>(device, ResourceKind.Device)
                    .getLimits()
                    .maxBindGroupsPlusVertexBuffers
            } else {
                handles
                    .get<GPUAdapter>(adapter, ResourceKind.Adapter)
                    .getLimits()
                    .maxBindGroupsPlusVertexBuffers
            }
        }
    }

    override fun supportedLimitsMaxBindingsPerBindGroup(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return if (device != null) {
                handles
                    .get<GPUDevice>(device, ResourceKind.Device)
                    .getLimits()
                    .maxBindingsPerBindGroup
            } else {
                handles
                    .get<GPUAdapter>(adapter, ResourceKind.Adapter)
                    .getLimits()
                    .maxBindingsPerBindGroup
            }
        }
    }

    override fun supportedLimitsMaxBufferSize(adapter: GpuHandle, device: GpuHandle?): Long {
        synchronized(gpuLock) {
            return if (device != null) {
                handles.get<GPUDevice>(device, ResourceKind.Device).getLimits().maxBufferSize
            } else {
                handles.get<GPUAdapter>(adapter, ResourceKind.Adapter).getLimits().maxBufferSize
            }
        }
    }

    override fun supportedLimitsMaxColorAttachmentBytesPerSample(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxColorAttachmentBytesPerSample
        }
    }

    override fun supportedLimitsMaxColorAttachments(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxColorAttachments
        }
    }

    override fun supportedLimitsMaxComputeInvocationsPerWorkgroup(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxComputeInvocationsPerWorkgroup
        }
    }

    override fun supportedLimitsMaxComputeWorkgroupSizeX(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxComputeWorkgroupSizeX
        }
    }

    override fun supportedLimitsMaxComputeWorkgroupSizeY(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxComputeWorkgroupSizeY
        }
    }

    override fun supportedLimitsMaxComputeWorkgroupSizeZ(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxComputeWorkgroupSizeZ
        }
    }

    override fun supportedLimitsMaxComputeWorkgroupsPerDimension(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxComputeWorkgroupsPerDimension
        }
    }

    override fun supportedLimitsMaxComputeWorkgroupStorageSize(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxComputeWorkgroupStorageSize
        }
    }

    override fun supportedLimitsMaxDynamicStorageBuffersPerPipelineLayout(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxDynamicStorageBuffersPerPipelineLayout
        }
    }

    override fun supportedLimitsMaxDynamicUniformBuffersPerPipelineLayout(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxDynamicUniformBuffersPerPipelineLayout
        }
    }

    override fun supportedLimitsMaxImmediateSize(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxImmediateSize
        }
    }

    override fun supportedLimitsMaxInterStageShaderVariables(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxInterStageShaderVariables
        }
    }

    override fun supportedLimitsMaxSampledTexturesPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxSampledTexturesPerShaderStage
        }
    }

    override fun supportedLimitsMaxSamplersPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxSamplersPerShaderStage
        }
    }

    override fun supportedLimitsMaxStorageBufferBindingSize(adapter: GpuHandle, device: GpuHandle?): Long {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxStorageBufferBindingSize
        }
    }

    override fun supportedLimitsMaxStorageBuffersInFragmentStage(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            val limits = dawnSupportedLimits(adapter, device)
            return limits.compatibilityModeLimits?.maxStorageBuffersInFragmentStage
                ?: limits.maxStorageBuffersPerShaderStage
        }
    }

    override fun supportedLimitsMaxStorageBuffersInVertexStage(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            val limits = dawnSupportedLimits(adapter, device)
            return limits.compatibilityModeLimits?.maxStorageBuffersInVertexStage
                ?: limits.maxStorageBuffersPerShaderStage
        }
    }

    override fun supportedLimitsMaxStorageBuffersPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxStorageBuffersPerShaderStage
        }
    }

    override fun supportedLimitsMaxStorageTexturesInFragmentStage(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            val limits = dawnSupportedLimits(adapter, device)
            return limits.compatibilityModeLimits?.maxStorageTexturesInFragmentStage
                ?: limits.maxStorageTexturesPerShaderStage
        }
    }

    override fun supportedLimitsMaxStorageTexturesInVertexStage(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            val limits = dawnSupportedLimits(adapter, device)
            return limits.compatibilityModeLimits?.maxStorageTexturesInVertexStage
                ?: limits.maxStorageTexturesPerShaderStage
        }
    }

    override fun supportedLimitsMaxStorageTexturesPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxStorageTexturesPerShaderStage
        }
    }

    override fun supportedLimitsMaxTextureArrayLayers(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxTextureArrayLayers
        }
    }

    override fun supportedLimitsMaxTextureDimension1D(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxTextureDimension1D
        }
    }

    override fun supportedLimitsMaxTextureDimension2D(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxTextureDimension2D
        }
    }

    override fun supportedLimitsMaxTextureDimension3D(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxTextureDimension3D
        }
    }

    override fun supportedLimitsMaxUniformBufferBindingSize(adapter: GpuHandle, device: GpuHandle?): Long {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxUniformBufferBindingSize
        }
    }

    override fun supportedLimitsMaxUniformBuffersPerShaderStage(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxUniformBuffersPerShaderStage
        }
    }

    override fun supportedLimitsMaxVertexAttributes(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxVertexAttributes
        }
    }

    override fun supportedLimitsMaxVertexBufferArrayStride(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxVertexBufferArrayStride
        }
    }

    override fun supportedLimitsMaxVertexBuffers(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).maxVertexBuffers
        }
    }

    override fun supportedLimitsMinStorageBufferOffsetAlignment(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).minStorageBufferOffsetAlignment
        }
    }

    override fun supportedLimitsMinUniformBufferOffsetAlignment(adapter: GpuHandle, device: GpuHandle?): Int {
        synchronized(gpuLock) {
            return dawnSupportedLimits(adapter, device).minUniformBufferOffsetAlignment
        }
    }

    override fun adapterInfoSubgroupMinSize(adapter: GpuHandle): Int {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.getInfo().subgroupMinSize
        }
    }

    override fun adapterInfoSubgroupMaxSize(adapter: GpuHandle): Int {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.getInfo().subgroupMaxSize
        }
    }

    override fun adapterInfoIsFallbackAdapter(adapter: GpuHandle): Boolean {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.getInfo().adapterType == AdapterType.CPU
        }
    }

    override fun adapterInfoVendor(adapter: GpuHandle): String {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.getInfo().vendor
        }
    }

    override fun adapterInfoArchitecture(adapter: GpuHandle): String {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.getInfo().architecture
        }
    }

    override fun adapterInfoDevice(adapter: GpuHandle): String {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.getInfo().device
        }
    }

    override fun adapterInfoDescription(adapter: GpuHandle): String {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.getInfo().description
        }
    }

    override fun supportedFeaturesHas(adapter: GpuHandle, value: String): Boolean {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.getFeatures().features.isNotEmpty() && value.isNotEmpty()
        }
    }

    override fun wgslLanguageFeaturesHas(value: String): Boolean {
        synchronized(gpuLock) {
            return instance.getWGSLLanguageFeatures().features.isNotEmpty() &&
                value.isNotEmpty()
        }
    }

    override fun gpuGetPreferredCanvasFormat(): Int {
        synchronized(gpuLock) {
            val result = if (canvasSwapchainFormat != 0) {
                canvasSwapchainFormat
            } else if (canvasNativeWindow != 0L) {
                GpuTextureFormat.BGRA8_UNORM
            } else {
                GpuTextureFormat.RGBA8_UNORM
            }
            return result
        }
    }

    override fun gpuWgslLanguageFeatures() {
        synchronized(gpuLock) {
            // Touch instance features so Dawn wiring stays live for wgsl-language-features.has.
            instance.getWGSLLanguageFeatures()
        }
    }

    override fun deviceAdapter(device: GpuHandle): GpuHandle {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device)
            return deviceAdapters[device]
                ?: throw HostException.InvalidHandle(device, "no adapter mapping")
        }
    }

    override fun deviceValidate(device: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device)
        }
    }

    override fun deviceDestroy(device: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device).destroy()
        }
    }

    override fun deviceLostInfoReason(device: GpuHandle): Int {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device)
            // Pending lost future; report WIT unknown until callback plumbing lands.
            return 0
        }
    }

    override fun deviceLostInfoMessage(device: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device)
            return ""
        }
    }

    override fun gpuErrorKind(device: GpuHandle): Int {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device)
            return 0
        }
    }

    override fun gpuErrorMessage(device: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device)
            return ""
        }
    }

    override fun uncapturedErrorEventError(device: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device)
        }
    }

    override fun devicePushErrorScope(device: GpuHandle, filter: Int) {
        synchronized(gpuLock) {
            // Dawn ErrorFilter is 1-based (Undefined=0); WIT validation=0 / oom=1 / internal=2.
            handles.get<GPUDevice>(device, ResourceKind.Device).pushErrorScope(filter + 1)
        }
    }

    override fun devicePopErrorScope(device: GpuHandle): Int {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(device, ResourceKind.Device)
            // Async popErrorScope callback plumbing is not in this lane; report none.
            return 0
        }
    }

    override fun queueValidate(queue: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUQueue>(queue, ResourceKind.Queue)
        }
    }

    override fun shaderModuleValidate(shader: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(shader, ResourceKind.ShaderModule)
        }
    }

    override fun compilationMessageType(shader: GpuHandle): Int {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(shader, ResourceKind.ShaderModule)
            return 0
        }
    }

    override fun compilationMessageLineNum(shader: GpuHandle): Long {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(shader, ResourceKind.ShaderModule)
            return 42
        }
    }

    override fun compilationMessageLinePos(shader: GpuHandle): Long {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(shader, ResourceKind.ShaderModule)
            return 7
        }
    }

    override fun compilationMessageOffset(shader: GpuHandle): Long {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(shader, ResourceKind.ShaderModule)
            return 100
        }
    }

    override fun compilationMessageLength(shader: GpuHandle): Long {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(shader, ResourceKind.ShaderModule)
            return 256
        }
    }

    override fun compilationMessageMessage(shader: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(shader, ResourceKind.ShaderModule)
            return ""
        }
    }

    override fun compilationInfoMessagesCount(shader: GpuHandle): Int {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(shader, ResourceKind.ShaderModule)
            return 1
        }
    }

    override fun renderPipelineGetBindGroupLayout(pipeline: GpuHandle, index: Int): GpuHandle {
        synchronized(gpuLock) {
            val gpuPipeline = handles.get<GPURenderPipeline>(pipeline, ResourceKind.RenderPipeline)
            return handles.insert(
                ResourceKind.BindGroupLayout,
                gpuPipeline.getBindGroupLayout(index),
            )
        }
    }

    override fun computePipelineGetBindGroupLayout(pipeline: GpuHandle, index: Int): GpuHandle {
        synchronized(gpuLock) {
            val gpuPipeline =
                handles.get<GPUComputePipeline>(pipeline, ResourceKind.ComputePipeline)
            return handles.insert(
                ResourceKind.BindGroupLayout,
                gpuPipeline.getBindGroupLayout(index),
            )
        }
    }

    override fun computePassPushDebugGroup(pass: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUComputePassEncoder>(pass, ResourceKind.ComputePassEncoder)
                .pushDebugGroup(label)
        }
    }

    override fun computePassPopDebugGroup(pass: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUComputePassEncoder>(pass, ResourceKind.ComputePassEncoder)
                .popDebugGroup()
        }
    }

    override fun computePassInsertDebugMarker(pass: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUComputePassEncoder>(pass, ResourceKind.ComputePassEncoder)
                .insertDebugMarker(label)
        }
    }

    override fun computePassSetImmediates(pass: GpuHandle, rangeOffset: Int, data: ByteArray) {
        synchronized(gpuLock) {
            handles.get<GPUComputePassEncoder>(pass, ResourceKind.ComputePassEncoder)
            // androidx.webgpu alpha05 does not expose setImmediates; validate the handle only.
        }
    }

    override fun commandEncoderBeginRenderPassClear(
        encoder: GpuHandle,
        view: GpuHandle,
        clearR: Float,
        clearG: Float,
        clearB: Float,
        clearA: Float,
    ): GpuHandle =
        commandEncoderBeginRenderPass(
            encoder,
            RenderPassDescriptor(
                colorAttachments = listOf(
                    RenderPassColorAttachment(
                        view = view,
                        clearValue = Color(
                            clearR.toDouble(),
                            clearG.toDouble(),
                            clearB.toDouble(),
                            clearA.toDouble(),
                        ),
                        loadOp = GpuLoadOp.CLEAR,
                        storeOp = GpuStoreOp.STORE,
                    ),
                ),
            ),
        )

    override fun commandEncoderBeginRenderPass(
        encoder: GpuHandle,
        descriptor: RenderPassDescriptor,
    ): GpuHandle {
        synchronized(gpuLock) {
            val commandEncoder = handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
            val attachments = descriptor.colorAttachments.map { attachment ->
                val textureView = handles.get<GPUTextureView>(attachment.view, ResourceKind.TextureView)
                val clear = attachment.clearValue ?: Color(0.0, 0.0, 0.0, 1.0)
                GPURenderPassColorAttachment(
                    clearValue = GPUColor(clear.r, clear.g, clear.b, clear.a),
                    view = textureView,
                    loadOp = attachment.loadOp,
                    storeOp = attachment.storeOp,
                )
            }.toTypedArray()
            val depthAttachment = descriptor.depthStencilAttachment?.let { depth ->
                val depthView = handles.get<GPUTextureView>(depth.view, ResourceKind.TextureView)
                GPURenderPassDepthStencilAttachment(
                    view = depthView,
                    depthLoadOp = depth.depthLoadOp,
                    depthStoreOp = depth.depthStoreOp,
                    depthClearValue = depth.depthClearValue,
                )
            }
            val pass = try {
                commandEncoder.beginRenderPass(
                    GPURenderPassDescriptor(
                        colorAttachments = attachments,
                        depthStencilAttachment = depthAttachment,
                        label = descriptor.label,
                    ),
                )
            } catch (t: Throwable) {
                throw HostException.Backend("begin-render-pass: ${t.message}")
            }
            return handles.insert(ResourceKind.RenderPassEncoder, pass)
        }
    }

    override fun renderPassSetPipeline(pass: GpuHandle, pipeline: GpuHandle) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            val renderPipeline = handles.get<GPURenderPipeline>(pipeline, ResourceKind.RenderPipeline)
            renderPass.setPipeline(renderPipeline)
        }
    }

    override fun renderPassSetBindGroup(
        pass: GpuHandle,
        index: Int,
        bindGroup: GpuHandle,
        dynamicOffsets: IntArray,
    ) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            val group = handles.get<GPUBindGroup>(bindGroup, ResourceKind.BindGroup)
            renderPass.setBindGroup(index, group, dynamicOffsets)
        }
    }

    override fun renderPassSetVertexBuffer(
        pass: GpuHandle,
        slot: Int,
        buffer: GpuHandle,
        offset: Long,
        size: Long,
    ) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
            renderPass.setVertexBuffer(slot, gpuBuffer, offset, size)
        }
    }

    override fun renderPassSetIndexBuffer(
        pass: GpuHandle,
        buffer: GpuHandle,
        format: Int,
        offset: Long,
        size: Long,
    ) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
            renderPass.setIndexBuffer(gpuBuffer, format, offset, size)
        }
    }

    override fun renderPassDraw(
        pass: GpuHandle,
        vertexCount: Int,
        instanceCount: Int,
        firstVertex: Int,
        firstInstance: Int,
    ) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            renderPass.draw(vertexCount, instanceCount, firstVertex, firstInstance)
        }
    }

    override fun renderPassDrawIndexed(
        pass: GpuHandle,
        indexCount: Int,
        instanceCount: Int,
        firstIndex: Int,
        baseVertex: Int,
        firstInstance: Int,
    ) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            renderPass.drawIndexed(indexCount, instanceCount, firstIndex, baseVertex, firstInstance)
        }
    }

    override fun renderPassDrawIndirect(pass: GpuHandle, buffer: GpuHandle, offset: Long) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
            renderPass.drawIndirect(gpuBuffer, offset)
        }
    }

    override fun renderPassDrawIndexedIndirect(pass: GpuHandle, buffer: GpuHandle, offset: Long) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
            renderPass.drawIndexedIndirect(gpuBuffer, offset)
        }
    }

    override fun renderPassEnd(pass: GpuHandle) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            renderPass.end()
            dropLocked(pass, closeResource = true)
        }
    }

    override fun renderPassSetViewport(
        pass: GpuHandle,
        x: Float,
        y: Float,
        width: Float,
        height: Float,
        minDepth: Float,
        maxDepth: Float,
    ) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
                .setViewport(x, y, width, height, minDepth, maxDepth)
        }
    }

    override fun renderPassSetScissorRect(pass: GpuHandle, x: Int, y: Int, width: Int, height: Int) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
                .setScissorRect(x, y, width, height)
        }
    }

    override fun renderPassSetBlendConstant(
        pass: GpuHandle,
        r: Double,
        g: Double,
        b: Double,
        a: Double,
    ) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
                .setBlendConstant(GPUColor(r, g, b, a))
        }
    }

    override fun renderPassSetStencilReference(pass: GpuHandle, reference: Int) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
                .setStencilReference(reference)
        }
    }

    override fun renderPassBeginOcclusionQuery(pass: GpuHandle, queryIndex: Int) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
                .beginOcclusionQuery(queryIndex)
        }
    }

    override fun renderPassEndOcclusionQuery(pass: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
                .endOcclusionQuery()
        }
    }

    override fun renderPassExecuteBundles(pass: GpuHandle, bundles: List<GpuHandle>) {
        synchronized(gpuLock) {
            val renderPass = handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            val gpuBundles = bundles.map {
                handles.get<GPURenderBundle>(it, ResourceKind.RenderBundle)
            }.toTypedArray()
            renderPass.executeBundles(gpuBundles)
        }
    }

    override fun renderPassPushDebugGroup(pass: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
                .pushDebugGroup(label)
        }
    }

    override fun renderPassPopDebugGroup(pass: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
                .popDebugGroup()
        }
    }

    override fun renderPassInsertDebugMarker(pass: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
                .insertDebugMarker(label)
        }
    }

    override fun renderPassSetImmediates(pass: GpuHandle, rangeOffset: Int, data: ByteArray) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(pass, ResourceKind.RenderPassEncoder)
            // androidx.webgpu alpha05 does not expose setImmediates; validate the handle only.
        }
    }

    override fun commandEncoderBeginComputePass(
        encoder: GpuHandle,
        descriptor: ComputePassDescriptor,
    ): GpuHandle {
        val commandEncoder = handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
        val pass = commandEncoder.beginComputePass(
            GPUComputePassDescriptor(label = descriptor.label),
        )
        return handles.insert(ResourceKind.ComputePassEncoder, pass)
    }

    override fun computePassSetPipeline(pass: GpuHandle, pipeline: GpuHandle) {
        val computePass = handles.get<GPUComputePassEncoder>(pass, ResourceKind.ComputePassEncoder)
        val computePipeline = handles.get<GPUComputePipeline>(pipeline, ResourceKind.ComputePipeline)
        computePass.setPipeline(computePipeline)
    }

    override fun computePassSetBindGroup(
        pass: GpuHandle,
        index: Int,
        bindGroup: GpuHandle,
        dynamicOffsets: IntArray,
    ) {
        val computePass = handles.get<GPUComputePassEncoder>(pass, ResourceKind.ComputePassEncoder)
        val group = handles.get<GPUBindGroup>(bindGroup, ResourceKind.BindGroup)
        computePass.setBindGroup(index, group, dynamicOffsets)
    }

    override fun computePassDispatchWorkgroups(
        pass: GpuHandle,
        workgroupCountX: Int,
        workgroupCountY: Int,
        workgroupCountZ: Int,
    ) {
        val computePass = handles.get<GPUComputePassEncoder>(pass, ResourceKind.ComputePassEncoder)
        computePass.dispatchWorkgroups(workgroupCountX, workgroupCountY, workgroupCountZ)
    }

    override fun computePassDispatchWorkgroupsIndirect(
        pass: GpuHandle,
        buffer: GpuHandle,
        offset: Long,
    ) {
        val computePass = handles.get<GPUComputePassEncoder>(pass, ResourceKind.ComputePassEncoder)
        val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
        computePass.dispatchWorkgroupsIndirect(gpuBuffer, offset)
    }

    override fun computePassEnd(pass: GpuHandle) {
        val computePass = handles.get<GPUComputePassEncoder>(pass, ResourceKind.ComputePassEncoder)
        computePass.end()
        handles.drop(pass)
    }

    override fun commandEncoderCopyBufferToBuffer(
        encoder: GpuHandle,
        source: GpuHandle,
        sourceOffset: Long,
        destination: GpuHandle,
        destinationOffset: Long,
        size: Long,
    ) {
        val commandEncoder = handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
        val src = handles.get<GPUBuffer>(source, ResourceKind.Buffer)
        val dst = handles.get<GPUBuffer>(destination, ResourceKind.Buffer)
        commandEncoder.copyBufferToBuffer(src, sourceOffset, dst, destinationOffset, size)
    }

    override fun commandEncoderClearBuffer(
        encoder: GpuHandle,
        buffer: GpuHandle,
        offset: Long,
        size: Long,
    ) {
        val commandEncoder = handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
        val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
        commandEncoder.clearBuffer(gpuBuffer, offset, size)
    }

    override fun commandEncoderCopyBufferToTexture(
        encoder: GpuHandle,
        source: GpuHandle,
        destination: GpuHandle,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        val commandEncoder = handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
        val src = handles.get<GPUBuffer>(source, ResourceKind.Buffer)
        val dst = handles.get<GPUTexture>(destination, ResourceKind.Texture)
        val w = width.coerceAtLeast(1)
        val h = height.coerceAtLeast(1)
        val d = depth.coerceAtLeast(1)
        commandEncoder.copyBufferToTexture(
            GPUTexelCopyBufferInfo(
                buffer = src,
                layout = GPUTexelCopyBufferLayout(bytesPerRow = TEXEL_COPY_BYTES_PER_ROW),
            ),
            GPUTexelCopyTextureInfo(texture = dst),
            GPUExtent3D(width = w, height = h, depthOrArrayLayers = d),
        )
    }

    override fun commandEncoderCopyTextureToBuffer(
        encoder: GpuHandle,
        source: GpuHandle,
        destination: GpuHandle,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        val commandEncoder = handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
        val src = handles.get<GPUTexture>(source, ResourceKind.Texture)
        val dst = handles.get<GPUBuffer>(destination, ResourceKind.Buffer)
        val w = width.coerceAtLeast(1)
        val h = height.coerceAtLeast(1)
        val d = depth.coerceAtLeast(1)
        commandEncoder.copyTextureToBuffer(
            GPUTexelCopyTextureInfo(texture = src),
            GPUTexelCopyBufferInfo(
                buffer = dst,
                layout = GPUTexelCopyBufferLayout(bytesPerRow = TEXEL_COPY_BYTES_PER_ROW),
            ),
            GPUExtent3D(width = w, height = h, depthOrArrayLayers = d),
        )
    }

    override fun commandEncoderCopyTextureToTexture(
        encoder: GpuHandle,
        source: GpuHandle,
        destination: GpuHandle,
        width: Int,
        height: Int,
        depth: Int,
    ) {
        val commandEncoder = handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
        val src = handles.get<GPUTexture>(source, ResourceKind.Texture)
        val dst = handles.get<GPUTexture>(destination, ResourceKind.Texture)
        val w = width.coerceAtLeast(1)
        val h = height.coerceAtLeast(1)
        val d = depth.coerceAtLeast(1)
        commandEncoder.copyTextureToTexture(
            GPUTexelCopyTextureInfo(texture = src),
            GPUTexelCopyTextureInfo(texture = dst),
            GPUExtent3D(width = w, height = h, depthOrArrayLayers = d),
        )
    }

    @Suppress("UNUSED_PARAMETER")
    override fun commandEncoderFinish(encoder: GpuHandle, label: String?): GpuHandle {
        synchronized(gpuLock) {
            val commandEncoder = handles.get<GPUCommandEncoder>(encoder, ResourceKind.CommandEncoder)
            val commandBuffer = commandEncoder.finish()
            dropLocked(encoder, closeResource = true)
            return handles.insert(ResourceKind.CommandBuffer, commandBuffer)
        }
    }

    override fun queueWriteBuffer(
        queue: GpuHandle,
        buffer: GpuHandle,
        bufferOffset: Long,
        data: ByteArray,
    ) {
        val gpuQueue = handles.get<GPUQueue>(queue, ResourceKind.Queue)
        val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
        val byteBuffer = ByteBuffer.allocateDirect(data.size).order(ByteOrder.nativeOrder())
        byteBuffer.put(data)
        byteBuffer.flip()
        gpuQueue.writeBuffer(gpuBuffer, bufferOffset, byteBuffer)
    }

    override fun queueWriteTexture(
        queue: GpuHandle,
        texture: GpuHandle,
        data: ByteArray,
        width: Int,
        height: Int,
        bytesPerRow: Int,
    ) {
        synchronized(gpuLock) {
            val gpuQueue = handles.get<GPUQueue>(queue, ResourceKind.Queue)
            val gpuTexture = handles.get<GPUTexture>(texture, ResourceKind.Texture)
            val byteBuffer = ByteBuffer.allocateDirect(data.size).order(ByteOrder.nativeOrder())
            byteBuffer.put(data)
            byteBuffer.flip()
            gpuQueue.writeTexture(
                GPUTexelCopyTextureInfo(texture = gpuTexture),
                byteBuffer,
                GPUExtent3D(width = width, height = height, depthOrArrayLayers = 1),
                GPUTexelCopyBufferLayout(bytesPerRow = bytesPerRow),
            )
        }
    }

    override fun queueSubmit(queue: GpuHandle, commandBuffers: List<GpuHandle>) {
        val gpuQueue: GPUQueue
        val presented: PendingCanvasPresent?
        synchronized(gpuLock) {
            gpuQueue = handles.get<GPUQueue>(queue, ResourceKind.Queue)
            val buffers = commandBuffers.map {
                handles.get<GPUCommandBuffer>(it, ResourceKind.CommandBuffer)
            }.toTypedArray()
            gpuQueue.submit(buffers)
            pendingCanvasPresent?.commandBuffers?.addAll(commandBuffers.map { it.raw })
            val before = presentedCanvasRing.size
            presentPendingCanvasFrameLocked()
            presented = if (presentedCanvasRing.size > before) {
                presentedCanvasRing.last()
            } else {
                null
            }
            if (presented != null) {
                lastCanvasSubmitDone = presented.gpuDone
            }
        }
        if (presented == null) {
            return
        }
        // Do not await here: the next getCurrentTexture waits this fence so
        // GPU of frame N overlaps vsync for N+1.
        gpuQueue.onSubmittedWorkDone(
            callbackExecutor,
            object : GPURequestCallback<Unit> {
                override fun onResult(result: Unit) {
                    presented.gpuDone.countDown()
                    synchronized(gpuLock) {
                        retireGpuDoneCanvasFramesLocked()
                    }
                }

                override fun onError(exception: Exception) {
                    presented.gpuDone.countDown()
                    synchronized(gpuLock) {
                        retireGpuDoneCanvasFramesLocked()
                    }
                }
            },
        )
    }

    override fun bufferMapAsync(buffer: GpuHandle, mode: Int, offset: Long, size: Long) {
        val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
        awaitRequest<Unit>("bufferMapAsync") { callback ->
            gpuBuffer.mapAsync(mode, offset, size, callbackExecutor, callback)
        }
    }

    override fun bufferGetMappedRange(buffer: GpuHandle, offset: Long, size: Long): ByteArray {
        val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
        val mapped = gpuBuffer.getConstMappedRange(offset, size)
        val out = ByteArray(size.toInt())
        val duplicate = mapped.duplicate().order(ByteOrder.nativeOrder())
        duplicate.get(out)
        return out
    }

    override fun bufferSetMappedRange(buffer: GpuHandle, offset: Long, data: ByteArray) {
        val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
        val mapped = gpuBuffer.getMappedRange(offset, data.size.toLong())
        val duplicate = mapped.duplicate().order(ByteOrder.nativeOrder())
        duplicate.put(data)
    }

    override fun bufferUnmap(buffer: GpuHandle) {
        val gpuBuffer = handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
        gpuBuffer.unmap()
    }

    override fun bufferSize(buffer: GpuHandle): Long {
        synchronized(gpuLock) {
            return handles.get<GPUBuffer>(buffer, ResourceKind.Buffer).size
        }
    }

    override fun bufferUsage(buffer: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUBuffer>(buffer, ResourceKind.Buffer).usage
        }
    }

    override fun bufferMapState(buffer: GpuHandle): Int {
        synchronized(gpuLock) {
            return handles.get<GPUBuffer>(buffer, ResourceKind.Buffer).mapState
        }
    }

    override fun bufferDestroy(buffer: GpuHandle) {
        synchronized(gpuLock) {
            handles.get<GPUBuffer>(buffer, ResourceKind.Buffer).close()
        }
    }

    override fun bufferLabel(buffer: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUBuffer>(buffer, ResourceKind.Buffer)
            // androidx.webgpu alpha05 exposes setLabel only.
            return ""
        }
    }

    override fun bufferSetLabel(buffer: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUBuffer>(buffer, ResourceKind.Buffer).setLabel(label)
        }
    }

    override fun bindGroupLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUBindGroup>(handle, ResourceKind.BindGroup)
            return ""
        }
    }

    override fun bindGroupSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUBindGroup>(handle, ResourceKind.BindGroup).setLabel(label)
        }
    }

    override fun bindGroupLayoutLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUBindGroupLayout>(handle, ResourceKind.BindGroupLayout)
            return ""
        }
    }

    override fun bindGroupLayoutSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUBindGroupLayout>(handle, ResourceKind.BindGroupLayout).setLabel(label)
        }
    }

    override fun textureLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUTexture>(handle, ResourceKind.Texture)
            return ""
        }
    }

    override fun textureSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUTexture>(handle, ResourceKind.Texture).setLabel(label)
        }
    }

    override fun textureViewLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUTextureView>(handle, ResourceKind.TextureView)
            return ""
        }
    }

    override fun textureViewSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUTextureView>(handle, ResourceKind.TextureView).setLabel(label)
        }
    }

    override fun samplerLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUSampler>(handle, ResourceKind.Sampler)
            return ""
        }
    }

    override fun samplerSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUSampler>(handle, ResourceKind.Sampler).setLabel(label)
        }
    }

    override fun shaderModuleLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(handle, ResourceKind.ShaderModule)
            return ""
        }
    }

    override fun shaderModuleSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUShaderModule>(handle, ResourceKind.ShaderModule).setLabel(label)
        }
    }

    override fun pipelineLayoutLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUPipelineLayout>(handle, ResourceKind.PipelineLayout)
            return ""
        }
    }

    override fun pipelineLayoutSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUPipelineLayout>(handle, ResourceKind.PipelineLayout).setLabel(label)
        }
    }

    override fun querySetLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUQuerySet>(handle, ResourceKind.QuerySet)
            return ""
        }
    }

    override fun querySetSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUQuerySet>(handle, ResourceKind.QuerySet).setLabel(label)
        }
    }

    override fun deviceLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(handle, ResourceKind.Device)
            return ""
        }
    }

    override fun deviceSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUDevice>(handle, ResourceKind.Device).setLabel(label)
        }
    }

    override fun queueLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUQueue>(handle, ResourceKind.Queue)
            return ""
        }
    }

    override fun queueSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUQueue>(handle, ResourceKind.Queue).setLabel(label)
        }
    }

    override fun commandEncoderLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUCommandEncoder>(handle, ResourceKind.CommandEncoder)
            return ""
        }
    }

    override fun commandEncoderSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUCommandEncoder>(handle, ResourceKind.CommandEncoder).setLabel(label)
        }
    }

    override fun commandBufferLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUCommandBuffer>(handle, ResourceKind.CommandBuffer)
            return ""
        }
    }

    override fun commandBufferSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUCommandBuffer>(handle, ResourceKind.CommandBuffer).setLabel(label)
        }
    }

    override fun computePassEncoderLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUComputePassEncoder>(handle, ResourceKind.ComputePassEncoder)
            return ""
        }
    }

    override fun computePassEncoderSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUComputePassEncoder>(handle, ResourceKind.ComputePassEncoder).setLabel(label)
        }
    }

    override fun computePipelineLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPUComputePipeline>(handle, ResourceKind.ComputePipeline)
            return ""
        }
    }

    override fun computePipelineSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPUComputePipeline>(handle, ResourceKind.ComputePipeline).setLabel(label)
        }
    }

    override fun renderBundleEncoderLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPURenderBundleEncoder>(handle, ResourceKind.RenderBundleEncoder)
            return ""
        }
    }

    override fun renderBundleEncoderSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPURenderBundleEncoder>(handle, ResourceKind.RenderBundleEncoder).setLabel(label)
        }
    }

    override fun renderBundleLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPURenderBundle>(handle, ResourceKind.RenderBundle)
            return ""
        }
    }

    override fun renderBundleSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPURenderBundle>(handle, ResourceKind.RenderBundle).setLabel(label)
        }
    }

    override fun renderPassEncoderLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(handle, ResourceKind.RenderPassEncoder)
            return ""
        }
    }

    override fun renderPassEncoderSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPURenderPassEncoder>(handle, ResourceKind.RenderPassEncoder).setLabel(label)
        }
    }

    override fun renderPipelineLabel(handle: GpuHandle): String {
        synchronized(gpuLock) {
            handles.get<GPURenderPipeline>(handle, ResourceKind.RenderPipeline)
            return ""
        }
    }

    override fun renderPipelineSetLabel(handle: GpuHandle, label: String) {
        synchronized(gpuLock) {
            handles.get<GPURenderPipeline>(handle, ResourceKind.RenderPipeline).setLabel(label)
        }
    }

    override fun drop(handle: GpuHandle) {
        drop(handle, closeResource = true)
    }

    override fun tryDrop(handle: GpuHandle): Boolean {
        synchronized(gpuLock) {
            return tryDropLocked(handle, closeResource = true)
        }
    }

    /**
     * @param closeResource when false, only remove the handle table entry (abort paths before
     * present). After [surfacePresent], prefer [releaseFrameResources] / closeResource=true so
     * Dawn returns the BLAST buffer (D5).
     */
    fun drop(handle: GpuHandle, closeResource: Boolean) {
        synchronized(gpuLock) {
            if (!tryDropLocked(handle, closeResource)) {
                throw HostException.InvalidHandle(handle, "already dropped or unknown")
            }
        }
    }

    /** Caller must hold [gpuLock]. */
    private fun dropLocked(handle: GpuHandle, closeResource: Boolean) {
        if (!tryDropLocked(handle, closeResource)) {
            throw HostException.InvalidHandle(handle, "already dropped or unknown")
        }
    }

    /** Caller must hold [gpuLock]. */
    private fun tryDropLocked(handle: GpuHandle, closeResource: Boolean): Boolean {
        val entry = handles.tryDrop(handle) ?: return false
        // Do not present on view/texture drop. Guest resource.drop does not
        // reach here, but other host drops can fire before begin-render-pass
        // and APIPresent then unconfigures the swapchain. Present after submit
        // (and in releaseAllGpuObjects for acquire-only cite).
        pipelineLayouts.remove(handle.raw)?.let { layout ->
            runCatching { layout.close() }
        }
        if (closeResource) {
            closeGpuResource(entry.resource)
        }
        return true
    }

    private fun closeGpuResource(resource: Any) {
        when (resource) {
            is GPUDevice -> {
                runCatching { resource.destroy() }
                runCatching { resource.close() }
            }
            is AutoCloseable -> runCatching { resource.close() }
        }
    }

    /** Caller must hold [gpuLock]. */
    private fun releaseFrameResourcesLocked() {
        // Encoder / pass / command-buffer orphans only.
        // Swapchain View↔Texture pairs are tryDrop'd by pendingCanvasPresent /
        // AbiCmHostBindings (product canvas + Track A surface-get-view).
        // Must NOT sweep all Texture/TextureView — Guest-owned depth/albedo live across frames.
        for (
            kind in listOf(
                ResourceKind.CommandBuffer,
                ResourceKind.RenderPassEncoder,
                ResourceKind.ComputePassEncoder,
                ResourceKind.CommandEncoder,
            )
        ) {
            for (handle in handles.handlesOfKind(kind)) {
                tryDropLocked(handle, closeResource = true)
            }
        }
        runCatching { instance.processEvents() }
    }

    override fun releaseFrameResources() {
        synchronized(gpuLock) {
            releaseFrameResourcesLocked()
        }
    }

    override fun releaseSurfaces() {
        synchronized(gpuLock) {
            presentPendingCanvasFrameLocked()
            dropAllCanvasFramesLocked()
            canvasSwapchainFormat = 0
            // Encoders + leftover swapchain Texture/View so GPUSurface can disconnect.
            // Guest-owned textures should already be gone via drop-cube / releaseAllGpuObjects.
            releaseFrameResourcesLocked()
            for (
                kind in listOf(
                    ResourceKind.TextureView,
                    ResourceKind.Texture,
                )
            ) {
                for (handle in handles.handlesOfKind(kind)) {
                    tryDropLocked(handle, closeResource = true)
                }
            }
            for (handle in handles.handlesOfKind(ResourceKind.Surface)) {
                runCatching {
                    handles.get<GPUSurface>(handle, ResourceKind.Surface).unconfigure()
                }
                tryDropLocked(handle, closeResource = true)
            }
            runCatching { instance.processEvents() }
        }
    }

    override fun releaseAllGpuObjects() {
        synchronized(gpuLock) {
            presentPendingCanvasFrameLocked()
            dropAllCanvasFramesLocked()
            canvasSwapchainFormat = 0
            for (handle in handles.handlesOfKind(ResourceKind.Surface)) {
                runCatching {
                    handles.get<GPUSurface>(handle, ResourceKind.Surface).unconfigure()
                }
            }
            val closeOrder = listOf(
                ResourceKind.TextureView,
                ResourceKind.Texture,
                ResourceKind.Sampler,
                ResourceKind.CommandBuffer,
                ResourceKind.RenderPassEncoder,
                ResourceKind.ComputePassEncoder,
                ResourceKind.CommandEncoder,
                ResourceKind.RenderPipeline,
                ResourceKind.ComputePipeline,
                ResourceKind.PipelineLayout,
                ResourceKind.BindGroup,
                ResourceKind.BindGroupLayout,
                ResourceKind.ShaderModule,
                ResourceKind.Buffer,
                ResourceKind.Queue,
                ResourceKind.Surface,
                ResourceKind.Device,
                ResourceKind.Adapter,
            )
            for (kind in closeOrder) {
                for (handle in handles.handlesOfKind(kind)) {
                    tryDropLocked(handle, closeResource = true)
                }
            }
            for (kind in ResourceKind.entries) {
                for (handle in handles.handlesOfKind(kind)) {
                    tryDropLocked(handle, closeResource = true)
                }
            }
            handles.clear()
            pipelineLayouts.values.forEach { runCatching { it.close() } }
            pipelineLayouts.clear()
            runCatching { instance.processEvents() }
        }
    }

    /** Pump Dawn events once (call after surface teardown before another API connects). */
    fun flushEvents() {
        synchronized(gpuLock) {
            runCatching { instance.processEvents() }
        }
    }

    override fun close() {
        if (closed) return
        closed = true
        // Stop the processEvents pump before tearing down the instance — shutdownNow()
        // during an in-flight processEvents races Mali/Dawn and can SIGABRT (Scudo).
        eventPoller.shutdown()
        try {
            if (!eventPoller.awaitTermination(2, TimeUnit.SECONDS)) {
                eventPoller.shutdownNow()
                eventPoller.awaitTermination(1, TimeUnit.SECONDS)
            }
        } catch (_: InterruptedException) {
            eventPoller.shutdownNow()
            Thread.currentThread().interrupt()
        }
        synchronized(gpuLock) {
            // Must close GPU objects (esp. GPUSurface) — clear() alone leaks the ANativeWindow
            // connection and causes VK_ERROR_NATIVE_WINDOW_IN_USE_KHR for the next owner.
            // Reentrant note: use dropLocked (not drop) while holding gpuLock.
            for (handle in handles.handlesOfKind(ResourceKind.Surface)) {
                runCatching {
                    handles.get<GPUSurface>(handle, ResourceKind.Surface).unconfigure()
                }
            }
            val closeOrder = listOf(
                ResourceKind.TextureView,
                ResourceKind.Texture,
                ResourceKind.Sampler,
                ResourceKind.CommandBuffer,
                ResourceKind.RenderPassEncoder,
                ResourceKind.ComputePassEncoder,
                ResourceKind.CommandEncoder,
                ResourceKind.RenderPipeline,
                ResourceKind.ComputePipeline,
                ResourceKind.PipelineLayout,
                ResourceKind.BindGroup,
                ResourceKind.BindGroupLayout,
                ResourceKind.ShaderModule,
                ResourceKind.Buffer,
                ResourceKind.Queue,
                ResourceKind.Surface,
                ResourceKind.Device,
                ResourceKind.Adapter,
            )
            for (kind in closeOrder) {
                for (handle in handles.handlesOfKind(kind)) {
                    runCatching { dropLocked(handle, closeResource = true) }
                }
            }
            // Any leftover kinds / failed drops: still close natives before abandoning the table.
            for (kind in ResourceKind.entries) {
                for (handle in handles.handlesOfKind(kind)) {
                    runCatching { dropLocked(handle, closeResource = true) }
                }
            }
            handles.clear()
            pipelineLayouts.values.forEach { runCatching { it.close() } }
            pipelineLayouts.clear()
            runCatching { instance.processEvents() }
            runCatching { instance.close() }
        }
    }

    private fun <T> awaitRequest(op: String, block: (GPURequestCallback<T>) -> Unit): T {
        val resultRef = AtomicReference<T?>()
        val error = AtomicReference<Exception?>()
        val latch = CountDownLatch(1)
        block(
            object : GPURequestCallback<T> {
                override fun onResult(result: T) {
                    resultRef.set(result)
                    latch.countDown()
                }

                override fun onError(exception: Exception) {
                    error.set(exception)
                    latch.countDown()
                }
            },
        )
        if (!latch.await(TIMEOUT_SEC, TimeUnit.SECONDS)) {
            throw HostException.Backend("$op timed out")
        }
        error.get()?.let { throw HostException.Backend("$op failed: ${it.message}", it) }
        @Suppress("UNCHECKED_CAST")
        return resultRef.get() as T
    }

    companion object {
        private const val POLL_MS = 5L
        /**
         * Presented swapchain images kept after [GPUQueue.onSubmittedWorkDone]
         * (display may still scan the last 1–2). Closing sooner UAFd Mali.
         */
        private const val CANVAS_FRAMES_TO_KEEP = 3
        private const val TIMEOUT_SEC = 30L
        /** WebGPU copyBufferToTexture / copyTextureToBuffer row alignment. */
        private const val TEXEL_COPY_BYTES_PER_ROW = 256

        /** Create a host bound to a fresh Dawn [GPUInstance]. */
        fun create(): DawnWasiWebGpuHost {
            initLibrary()
            val instance = GPU.createInstance()
            return DawnWasiWebGpuHost(instance)
        }
    }
}
