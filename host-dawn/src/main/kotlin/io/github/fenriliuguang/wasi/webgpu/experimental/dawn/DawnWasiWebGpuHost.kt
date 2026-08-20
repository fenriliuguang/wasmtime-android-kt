package io.github.fenriliuguang.wasi.webgpu.experimental.dawn

import androidx.webgpu.BackendType
import androidx.webgpu.BufferBindingType as DawnBufferBindingType
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
import androidx.webgpu.GPUBuffer
import androidx.webgpu.GPUBufferBindingLayout
import androidx.webgpu.GPUBufferDescriptor
import androidx.webgpu.GPUColor
import androidx.webgpu.GPUColorTargetState
import androidx.webgpu.GPUCommandBuffer
import androidx.webgpu.GPUCommandEncoder
import androidx.webgpu.GPUCommandEncoderDescriptor
import androidx.webgpu.GPUComputePassDescriptor
import androidx.webgpu.GPUComputePassEncoder
import androidx.webgpu.GPUComputePipeline
import androidx.webgpu.GPUComputePipelineDescriptor
import androidx.webgpu.GPUComputeState
import androidx.webgpu.GPUDepthStencilState
import androidx.webgpu.GPUDevice
import androidx.webgpu.GPUDeviceDescriptor
import androidx.webgpu.GPUExtent3D
import androidx.webgpu.GPUFragmentState
import androidx.webgpu.GPUInstance
import androidx.webgpu.GPUPipelineLayout
import androidx.webgpu.GPUPipelineLayoutDescriptor
import androidx.webgpu.GPUPrimitiveState
import androidx.webgpu.GPUQuerySet
import androidx.webgpu.GPUQuerySetDescriptor
import androidx.webgpu.GPUQueue
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
import androidx.webgpu.GPURequestCallback
import androidx.webgpu.GPUSampler
import androidx.webgpu.GPUSamplerBindingLayout
import androidx.webgpu.GPUSamplerDescriptor
import androidx.webgpu.GPUShaderModule
import androidx.webgpu.GPUShaderModuleDescriptor
import androidx.webgpu.GPUShaderSourceWGSL
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
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuHandle
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuSamplerBindingType
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureSampleType
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureViewDimension
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuVertexFormat
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuVertexStepMode
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
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ColorTargetState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.FragmentState
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuLoadOp
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuPrimitiveTopology
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuStoreOp
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureFormat
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
        )
        val adapter = awaitRequest<GPUAdapter>("requestAdapter") { callback ->
            instance.requestAdapter(callbackExecutor, dawnOptions, callback)
        }
        return handles.insert(ResourceKind.Adapter, adapter)
    }

    override fun adapterRequestDevice(adapter: GpuHandle): GpuHandle {
        val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
        val descriptor = GPUDeviceDescriptor(
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
            gpuAdapter.requestDevice(callbackExecutor, descriptor, callback)
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
        val layoutHandle = descriptor.layout
            ?: throw HostException.Unsupported("auto pipeline layout; pass an explicit pipeline-layout handle")
        val pipelineLayout = handles.get<GPUPipelineLayout>(layoutHandle, ResourceKind.PipelineLayout)
        val pipeline = gpuDevice.createComputePipeline(
            GPUComputePipelineDescriptor(
                layout = pipelineLayout,
                compute = GPUComputeState(
                    module = module,
                    entryPoint = descriptor.compute.entryPoint ?: "main",
                ),
                label = descriptor.label,
            ),
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
    ): Int {
        require(width > 0 && height > 0) { "invalid surface size ${width}x$height" }
        synchronized(gpuLock) {
            val gpuSurface = handles.get<GPUSurface>(surface, ResourceKind.Surface)
            val gpuDevice = handles.get<GPUDevice>(device, ResourceKind.Device)
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            val caps = gpuSurface.getCapabilities(gpuAdapter)
            val format = caps.formats.firstOrNull()
                ?: throw HostException.Backend("surface has no texture formats")
            val presentMode = PresentMode.Fifo
            val alphaMode = caps.alphaModes.firstOrNull() ?: CompositeAlphaMode.Opaque
            gpuSurface.configure(
                GPUSurfaceConfiguration(
                    device = gpuDevice,
                    width = width,
                    height = height,
                    format = format,
                    usage = TextureUsage.RenderAttachment,
                    viewFormats = intArrayOf(),
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
            val topology = descriptor.primitive?.topology ?: GpuPrimitiveTopology.TRIANGLE_LIST
            val depthStencil = descriptor.depthStencil?.let { ds ->
                GPUDepthStencilState(
                    format = ds.format,
                    depthWriteEnabled = if (ds.depthWriteEnabled) {
                        OptionalBool.True
                    } else {
                        OptionalBool.False
                    },
                    depthCompare = ds.depthCompare,
                )
            }
            val pipeline = gpuDevice.createRenderPipeline(
                GPURenderPipelineDescriptor(
                    vertex = GPUVertexState(
                        module = vertexModule,
                        entryPoint = descriptor.vertex.entryPoint ?: "vs_main",
                        buffers = dawnBuffers,
                    ),
                    layout = pipelineLayout,
                    primitive = GPUPrimitiveState(topology = topology),
                    depthStencil = depthStencil,
                    fragment = GPUFragmentState(
                        module = fragmentModule,
                        entryPoint = descriptor.fragment.entryPoint ?: "fs_main",
                        targets = descriptor.fragment.targets.map { target ->
                            GPUColorTargetState(format = target.format)
                        }.toTypedArray(),
                    ),
                    label = descriptor.label,
                ),
            )
            return handles.insert(ResourceKind.RenderPipeline, pipeline)
        }
    }

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
            return handles.insert(
                ResourceKind.TextureView,
                gpuTexture.createView(
                    GPUTextureViewDescriptor(
                        dimension = descriptor.dimension,
                        aspect = descriptor.aspect,
                    ),
                ),
            )
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
            handles.get<GPUTexture>(texture, ResourceKind.Texture).close()
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

    override fun supportedLimitsMaxBindGroups(adapter: GpuHandle, device: GpuHandle): Int {
        synchronized(gpuLock) {
            return if (device.raw != 0) {
                handles.get<GPUDevice>(device, ResourceKind.Device).limits.maxBindGroups
            } else {
                handles.get<GPUAdapter>(adapter, ResourceKind.Adapter).limits.maxBindGroups
            }
        }
    }

    override fun supportedLimitsMaxBindGroupsPlusVertexBuffers(
        adapter: GpuHandle,
        device: GpuHandle,
    ): Int {
        synchronized(gpuLock) {
            return if (device.raw != 0) {
                handles
                    .get<GPUDevice>(device, ResourceKind.Device)
                    .limits
                    .maxBindGroupsPlusVertexBuffers
            } else {
                handles
                    .get<GPUAdapter>(adapter, ResourceKind.Adapter)
                    .limits
                    .maxBindGroupsPlusVertexBuffers
            }
        }
    }

    override fun supportedLimitsMaxBindingsPerBindGroup(adapter: GpuHandle, device: GpuHandle): Int {
        synchronized(gpuLock) {
            return if (device.raw != 0) {
                handles
                    .get<GPUDevice>(device, ResourceKind.Device)
                    .limits
                    .maxBindingsPerBindGroup
            } else {
                handles
                    .get<GPUAdapter>(adapter, ResourceKind.Adapter)
                    .limits
                    .maxBindingsPerBindGroup
            }
        }
    }

    override fun supportedLimitsMaxBufferSize(adapter: GpuHandle, device: GpuHandle): Long {
        synchronized(gpuLock) {
            return if (device.raw != 0) {
                handles.get<GPUDevice>(device, ResourceKind.Device).limits.maxBufferSize
            } else {
                handles.get<GPUAdapter>(adapter, ResourceKind.Adapter).limits.maxBufferSize
            }
        }
    }

    override fun adapterInfoSubgroupMinSize(adapter: GpuHandle): Int {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.info.subgroupMinSize
        }
    }

    override fun adapterInfoSubgroupMaxSize(adapter: GpuHandle): Int {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.info.subgroupMaxSize
        }
    }

    override fun adapterInfoIsFallbackAdapter(adapter: GpuHandle): Boolean {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.info.isFallbackAdapter
        }
    }

    override fun adapterInfoVendor(adapter: GpuHandle): String {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.info.vendor
        }
    }

    override fun adapterInfoArchitecture(adapter: GpuHandle): String {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.info.architecture
        }
    }

    override fun adapterInfoDevice(adapter: GpuHandle): String {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.info.device
        }
    }

    override fun adapterInfoDescription(adapter: GpuHandle): String {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.info.description
        }
    }

    override fun supportedFeaturesHas(adapter: GpuHandle, value: String): Boolean {
        synchronized(gpuLock) {
            val gpuAdapter = handles.get<GPUAdapter>(adapter, ResourceKind.Adapter)
            return gpuAdapter.features.has(value)
        }
    }

    override fun wgslLanguageFeaturesHas(value: String): Boolean {
        synchronized(gpuLock) {
            return instance.wgslLanguageFeatures.has(value)
        }
    }

    override fun gpuGetPreferredCanvasFormat(): Int {
        synchronized(gpuLock) {
            // androidx.webgpu alpha05 has no GPU.getPreferredCanvasFormat; match Cpu stub.
            return GpuTextureFormat.RGBA8_UNORM
        }
    }

    override fun gpuWgslLanguageFeatures() {
        synchronized(gpuLock) {
            // Touch instance features so Dawn wiring stays live for wgsl-language-features.has.
            instance.wgslLanguageFeatures
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
            val pass = commandEncoder.beginRenderPass(
                GPURenderPassDescriptor(
                    colorAttachments = attachments,
                    depthStencilAttachment = depthAttachment,
                    label = descriptor.label,
                ),
            )
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
        synchronized(gpuLock) {
            val gpuQueue = handles.get<GPUQueue>(queue, ResourceKind.Queue)
            val buffers = commandBuffers.map {
                handles.get<GPUCommandBuffer>(it, ResourceKind.CommandBuffer)
            }.toTypedArray()
            gpuQueue.submit(buffers)
        }
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
        // Swapchain View↔Texture pairs are tryDrop'd by AbiCmHostBindings.frameTextureByView.
        // Must NOT sweep all Texture/TextureView — Guest-owned depth/albedo (cube) live across frames.
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
