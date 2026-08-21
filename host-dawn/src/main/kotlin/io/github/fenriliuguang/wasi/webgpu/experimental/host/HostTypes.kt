package io.github.fenriliuguang.wasi.webgpu.experimental.host

/**
 * L2 descriptor / flag types for the compute + minimal surface/render subset.
 *
 * Names follow wasi:webgpu WIT; values are chosen for Kotlin ergonomics.
 * This is NOT a full kotlin-webgpu client shim.
 */

object GpuBufferUsage {
    const val MAP_READ: Int = 1 shl 0
    const val MAP_WRITE: Int = 1 shl 1
    const val COPY_SRC: Int = 1 shl 2
    const val COPY_DST: Int = 1 shl 3
    const val INDEX: Int = 1 shl 4
    const val VERTEX: Int = 1 shl 5
    const val UNIFORM: Int = 1 shl 6
    const val STORAGE: Int = 1 shl 7
    const val INDIRECT: Int = 1 shl 8
    const val QUERY_RESOLVE: Int = 1 shl 9
}

object GpuShaderStage {
    const val VERTEX: Int = 1 shl 0
    const val FRAGMENT: Int = 1 shl 1
    const val COMPUTE: Int = 1 shl 2
}

object GpuMapMode {
    const val READ: Int = 1 shl 0
    const val WRITE: Int = 1 shl 1
}

/** WIT `gpu-buffer-map-state` ordinals (unmapped / pending / mapped). */
object GpuBufferMapState {
    const val UNMAPPED: Int = 0
    const val PENDING: Int = 1
    const val MAPPED: Int = 2
}

/** WIT `gpu-query-type` ordinals (occlusion / timestamp). */
object GpuQueryType {
    const val OCCLUSION: Int = 0
    const val TIMESTAMP: Int = 1
}

/** WebGPU GPUTextureUsage bitfield (same bits as wasi:webgpu / Dawn TextureUsage). */
object GpuTextureUsage {
    const val COPY_SRC: Int = 1 shl 0
    const val COPY_DST: Int = 1 shl 1
    const val TEXTURE_BINDING: Int = 1 shl 2
    const val STORAGE_BINDING: Int = 1 shl 3
    const val RENDER_ATTACHMENT: Int = 1 shl 4
}

/**
 * Dawn / WebGPU TextureDimension ordinals used by L2 (pass-through to androidx.webgpu).
 * Prefer [D2] for slice D create-texture.
 */
object GpuTextureDimension {
    const val D1: Int = 0x00000001
    const val D2: Int = 0x00000002
    const val D3: Int = 0x00000003
}

/**
 * Common Dawn TextureFormat values (pass-through).
 * Values match `androidx.webgpu` 1.0.0-alpha05.
 */
object GpuTextureFormat {
    /** Dawn `TextureFormat.Undefined`. */
    const val UNDEFINED: Int = 0x00000000
    /** `androidx.webgpu.TextureFormat.RGBA8Unorm` (alpha05 = 0x16; older 0x12 was wrong). */
    const val RGBA8_UNORM: Int = 0x00000016
    /** `androidx.webgpu.TextureFormat.Depth24Plus` */
    const val DEPTH24_PLUS: Int = 0x0000002e
}

/**
 * Dawn AddressMode pass-through (`androidx.webgpu.AddressMode`).
 * Undefined=0 lets Dawn apply ClampToEdge.
 */
object GpuAddressMode {
    const val UNDEFINED: Int = 0x00000000
    const val CLAMP_TO_EDGE: Int = 0x00000001
    const val REPEAT: Int = 0x00000002
    const val MIRROR_REPEAT: Int = 0x00000003
}

/**
 * Dawn FilterMode pass-through (`androidx.webgpu.FilterMode`).
 * Undefined=0 lets Dawn apply Nearest.
 */
object GpuFilterMode {
    const val UNDEFINED: Int = 0x00000000
    const val NEAREST: Int = 0x00000001
    const val LINEAR: Int = 0x00000002
}

/**
 * Dawn MipmapFilterMode pass-through (`androidx.webgpu.MipmapFilterMode`).
 * Undefined=0 lets Dawn apply Nearest.
 */
object GpuMipmapFilterMode {
    const val UNDEFINED: Int = 0x00000000
    const val NEAREST: Int = 0x00000001
    const val LINEAR: Int = 0x00000002
}

/** SamplerBindingType ordinals (wasi enum order). */
object GpuSamplerBindingType {
    const val FILTERING: Int = 0
    const val NON_FILTERING: Int = 1
    const val COMPARISON: Int = 2
}

/** TextureSampleType ordinals (wasi enum order). */
object GpuTextureSampleType {
    const val FLOAT: Int = 0
    const val UNFILTERABLE_FLOAT: Int = 1
    const val DEPTH: Int = 2
    const val SINT: Int = 3
    const val UINT: Int = 4
}

/** TextureViewDimension ordinals (wasi enum order; d2 default). */
object GpuTextureViewDimension {
    const val D1: Int = 0
    const val D2: Int = 1
    const val D2_ARRAY: Int = 2
    const val CUBE: Int = 3
    const val CUBE_ARRAY: Int = 4
    const val D3: Int = 5
}

data class Extent3D(
    val width: Int,
    val height: Int = 1,
    val depthOrArrayLayers: Int = 1,
)

data class TextureDescriptor(
    val size: Extent3D,
    val format: Int,
    val usage: Int,
    val mipLevelCount: Int = 1,
    val sampleCount: Int = 1,
    val dimension: Int = GpuTextureDimension.D2,
    val label: String? = null,
)

/** Sampler descriptor; Dawn ints (0 = Undefined → host default). Lod defaults match WebGPU. */
data class SamplerDescriptor(
    val label: String? = null,
    val magFilter: Int = GpuFilterMode.UNDEFINED,
    val minFilter: Int = GpuFilterMode.UNDEFINED,
    val addressModeU: Int = GpuAddressMode.UNDEFINED,
    val addressModeV: Int = GpuAddressMode.UNDEFINED,
    val addressModeW: Int = GpuAddressMode.UNDEFINED,
    val mipmapFilter: Int = GpuMipmapFilterMode.UNDEFINED,
    val lodMinClamp: Float = 0f,
    val lodMaxClamp: Float = 32f,
    val compare: Int = GpuCompareFunction.UNDEFINED,
)

/**
 * Dawn TextureAspect pass-through (`androidx.webgpu.TextureAspect`).
 * Undefined=0 lets Dawn apply All.
 */
object GpuTextureAspect {
    const val UNDEFINED: Int = 0x00000000
    const val ALL: Int = 0x00000001
    const val STENCIL_ONLY: Int = 0x00000002
    const val DEPTH_ONLY: Int = 0x00000003
}

/**
 * Dawn IndexFormat pass-through (`androidx.webgpu.IndexFormat`).
 * Undefined=0 is invalid for set-index-buffer.
 */
object GpuIndexFormat {
    const val UNDEFINED: Int = 0x00000000
    const val UINT16: Int = 0x00000001
    const val UINT32: Int = 0x00000002
}

/**
 * Dawn TextureViewDimension pass-through (`androidx.webgpu.TextureViewDimension`).
 * Distinct from [GpuTextureViewDimension] (WIT ordinals). Undefined=0 lets Dawn apply 2D.
 */
object GpuDawnTextureViewDimension {
    const val UNDEFINED: Int = 0x00000000
    const val D1: Int = 0x00000001
    const val D2: Int = 0x00000002
    const val D2_ARRAY: Int = 0x00000003
    const val CUBE: Int = 0x00000004
    const val CUBE_ARRAY: Int = 0x00000005
    const val D3: Int = 0x00000006
}

/** Texture-view descriptor; Dawn ints (0 = Undefined → host default).
 * `mipLevelCount` / `arrayLayerCount` `-1` = Dawn UNDEFINED (all remaining). */
data class TextureViewDescriptor(
    val dimension: Int = GpuDawnTextureViewDimension.UNDEFINED,
    val aspect: Int = GpuTextureAspect.UNDEFINED,
    val format: Int = GpuTextureFormat.UNDEFINED,
    val baseMipLevel: Int = 0,
    val mipLevelCount: Int = -1,
    val baseArrayLayer: Int = 0,
    val arrayLayerCount: Int = -1,
)

data class PipelineLayoutDescriptor(
    val bindGroupLayouts: List<GpuHandle>,
    val label: String? = null,
)

data class SamplerBindingLayout(
    val type: Int = GpuSamplerBindingType.FILTERING,
)

data class TextureBindingLayout(
    val sampleType: Int = GpuTextureSampleType.FLOAT,
    val viewDimension: Int = GpuTextureViewDimension.D2,
    val multisampled: Boolean = false,
)

data class RequestAdapterOptions(
    val powerPreference: PowerPreference = PowerPreference.Undefined,
    val forceFallbackAdapter: Boolean = false,
    val featureLevel: String? = null,
)

/** Guest `gpu-device-descriptor` leftover fields (required-limits map + label). */
data class DeviceDescriptor(
    val requiredLimits: Map<String, Long?> = emptyMap(),
    val label: String? = null,
)

enum class PowerPreference {
    Undefined,
    LowPower,
    HighPerformance,
}

data class BufferDescriptor(
    val size: Long,
    val usage: Int,
    val mappedAtCreation: Boolean = false,
    val label: String? = null,
)

/**
 * WebGPU / Dawn GPUVertexFormat numeric values (`androidx.webgpu.VertexFormat`).
 * Only formats needed by the experimental render subset are listed.
 */
object GpuVertexFormat {
    /** `androidx.webgpu.VertexFormat.Float32x2` */
    const val FLOAT32X2: Int = 0x0000001d
    /** `androidx.webgpu.VertexFormat.Float32x3` */
    const val FLOAT32X3: Int = 0x0000001e
}

/**
 * Dawn CompareFunction pass-through (`androidx.webgpu.CompareFunction`).
 */
object GpuCompareFunction {
    const val UNDEFINED: Int = 0x00000000
    const val NEVER: Int = 0x00000001
    const val LESS: Int = 0x00000002
    const val EQUAL: Int = 0x00000003
    const val LESS_EQUAL: Int = 0x00000004
    const val GREATER: Int = 0x00000005
    const val NOT_EQUAL: Int = 0x00000006
    const val GREATER_EQUAL: Int = 0x00000007
    const val ALWAYS: Int = 0x00000008
}

/**
 * Dawn OptionalBool pass-through (`androidx.webgpu.OptionalBool`) for depthWriteEnabled.
 */
object GpuOptionalBool {
    const val FALSE: Int = 0x00000000
    const val TRUE: Int = 0x00000001
}

/**
 * WebGPU / Dawn GPUVertexStepMode numeric values (`androidx.webgpu.VertexStepMode`).
 */
object GpuVertexStepMode {
    /** `androidx.webgpu.VertexStepMode.Vertex` */
    const val VERTEX: Int = 0x00000001
    /** `androidx.webgpu.VertexStepMode.Instance` */
    const val INSTANCE: Int = 0x00000002
}

data class VertexAttribute(
    val format: Int,
    val offset: Long,
    val shaderLocation: Int,
)

data class VertexBufferLayout(
    val arrayStride: Long,
    val stepMode: Int = GpuVertexStepMode.VERTEX,
    val attributes: List<VertexAttribute>,
)

data class ShaderModuleDescriptor(
    val code: String,
    val label: String? = null,
)

enum class BufferBindingType {
    Uniform,
    Storage,
    ReadOnlyStorage,
}

data class BufferBindingLayout(
    val type: BufferBindingType = BufferBindingType.Uniform,
    val hasDynamicOffset: Boolean = false,
    val minBindingSize: Long = 0,
)

data class BindGroupLayoutEntry(
    val binding: Int,
    val visibility: Int,
    val buffer: BufferBindingLayout? = null,
    val sampler: SamplerBindingLayout? = null,
    val texture: TextureBindingLayout? = null,
)

data class BindGroupLayoutDescriptor(
    val entries: List<BindGroupLayoutEntry>,
    val label: String? = null,
)

data class BufferBinding(
    val buffer: GpuHandle,
    val offset: Long = 0,
    val size: Long? = null,
)

/** One of buffer / sampler / texture-view (WebGPU GPUBindingResource subset). */
sealed class BindingResource {
    data class Buffer(val binding: BufferBinding) : BindingResource()
    data class Sampler(val sampler: GpuHandle) : BindingResource()
    data class TextureView(val view: GpuHandle) : BindingResource()
}

data class BindGroupEntry(
    val binding: Int,
    val resource: BindingResource,
) {
    constructor(binding: Int, buffer: BufferBinding) : this(binding, BindingResource.Buffer(buffer))
}

data class BindGroupDescriptor(
    val layout: GpuHandle,
    val entries: List<BindGroupEntry>,
    val label: String? = null,
)

data class ProgrammableStage(
    val module: GpuHandle,
    val entryPoint: String? = null,
    val constants: Map<String, Double> = emptyMap(),
)

/**
 * [layout] is a [ResourceKind.PipelineLayout] handle (slice D).
 * Pass null only if the backend supports auto layout (this repo does not).
 */
data class ComputePipelineDescriptor(
    val compute: ProgrammableStage,
    val layout: GpuHandle? = null,
    val label: String? = null,
)

data class CommandEncoderDescriptor(
    val label: String? = null,
)

data class ComputePassDescriptor(
    val label: String? = null,
)

/** Status from [WasiWebGpuHost.surfaceGetCurrentTexture] (Dawn surface acquire). */
enum class SurfaceTextureStatus {
    SuccessOptimal,
    SuccessSuboptimal,
    Timeout,
    Outdated,
    Lost,
    Error,
}

data class SurfaceTextureResult(
    val status: SurfaceTextureStatus,
    val texture: GpuHandle?,
)

/** Dawn LoadOp pass-through. */
object GpuLoadOp {
    const val UNDEFINED: Int = 0
    const val LOAD: Int = 1
    const val CLEAR: Int = 2
}

/** Dawn StoreOp pass-through. */
object GpuStoreOp {
    const val UNDEFINED: Int = 0
    const val STORE: Int = 1
    const val DISCARD: Int = 2
}

/** Dawn PrimitiveTopology pass-through. */
object GpuPrimitiveTopology {
    const val UNDEFINED: Int = 0
    const val POINT_LIST: Int = 1
    const val LINE_LIST: Int = 2
    const val LINE_STRIP: Int = 3
    const val TRIANGLE_LIST: Int = 4
    const val TRIANGLE_STRIP: Int = 5
}

/** Dawn CullMode pass-through. Undefined=0, None=1, Front=2, Back=3. */
object GpuCullMode {
    const val UNDEFINED: Int = 0
    const val NONE: Int = 1
    const val FRONT: Int = 2
    const val BACK: Int = 3
}

/** Dawn FrontFace pass-through. Undefined=0, CCW=1, CW=2. */
object GpuFrontFace {
    const val UNDEFINED: Int = 0
    const val CCW: Int = 1
    const val CW: Int = 2
}

data class BlendComponent(
    val operation: Int = 0,
    val srcFactor: Int = 0,
    val dstFactor: Int = 0,
)

data class BlendState(
    val color: BlendComponent = BlendComponent(),
    val alpha: BlendComponent = BlendComponent(),
)

data class Color(
    val r: Double,
    val g: Double,
    val b: Double,
    val a: Double,
)

data class ColorTargetState(
    val format: Int,
    val blend: BlendState? = null,
)

data class VertexState(
    val module: GpuHandle,
    val entryPoint: String? = null,
    val buffers: List<VertexBufferLayout> = emptyList(),
    val constants: Map<String, Double> = emptyMap(),
)

data class FragmentState(
    val module: GpuHandle,
    val entryPoint: String? = null,
    val targets: List<ColorTargetState>,
    val constants: Map<String, Double> = emptyMap(),
)

data class PrimitiveState(
    val topology: Int = GpuPrimitiveTopology.TRIANGLE_LIST,
    val cullMode: Int = GpuCullMode.UNDEFINED,
    val stripIndexFormat: Int = GpuIndexFormat.UNDEFINED,
    val frontFace: Int = GpuFrontFace.UNDEFINED,
)

data class MultisampleState(
    val count: Int = 1,
    val mask: Int = -1,
    val alphaToCoverageEnabled: Boolean = false,
)

data class DepthStencilState(
    val format: Int,
    val depthWriteEnabled: Boolean = true,
    /** Dawn CompareFunction pass-through ([GpuCompareFunction]). */
    val depthCompare: Int = GpuCompareFunction.LESS,
)

data class RenderPipelineDescriptor(
    val vertex: VertexState,
    val fragment: FragmentState,
    /** [ResourceKind.PipelineLayout] handle. */
    val layout: GpuHandle,
    val primitive: PrimitiveState? = PrimitiveState(),
    val depthStencil: DepthStencilState? = null,
    val multisample: MultisampleState? = null,
    val label: String? = null,
)

fun primitiveStateFromDescribed(primitive: IntArray): PrimitiveState {
    val topology = primitive.getOrElse(0) { 0 }.let { if (it != 0) it else GpuPrimitiveTopology.TRIANGLE_LIST }
    return PrimitiveState(
        topology = topology,
        stripIndexFormat = primitive.getOrElse(1) { 0 },
        frontFace = primitive.getOrElse(2) { 0 },
        cullMode = primitive.getOrElse(3) { 0 },
    )
}

fun multisampleStateFromDescribed(multisample: IntArray): MultisampleState? {
    if (multisample.isEmpty()) return null
    val count = multisample.getOrElse(0) { 0 }
    val hasMask = multisample.getOrElse(1) { 0 }
    val mask = multisample.getOrElse(2) { 0 }
    val alpha = multisample.getOrElse(3) { -1 }
    if (count == 0 && hasMask == 0 && alpha < 0) return null
    return MultisampleState(
        count = if (count != 0) count else 1,
        mask = if (hasMask != 0) mask else -1,
        alphaToCoverageEnabled = alpha == 1,
    )
}

fun blendStateFromDescribed(blend: IntArray, targetIndex: Int = 0): BlendState? {
    val base = targetIndex * 7
    if (blend.size < base + 7 || blend[base] == 0) return null
    return BlendState(
        color = BlendComponent(
            operation = blend[base + 1],
            srcFactor = blend[base + 2],
            dstFactor = blend[base + 3],
        ),
        alpha = BlendComponent(
            operation = blend[base + 4],
            srcFactor = blend[base + 5],
            dstFactor = blend[base + 6],
        ),
    )
}

data class RenderPassColorAttachment(
    val view: GpuHandle,
    val clearValue: Color? = null,
    val loadOp: Int = GpuLoadOp.CLEAR,
    val storeOp: Int = GpuStoreOp.STORE,
)

data class RenderPassDepthStencilAttachment(
    val view: GpuHandle,
    val depthClearValue: Float = 1f,
    val depthLoadOp: Int = GpuLoadOp.CLEAR,
    val depthStoreOp: Int = GpuStoreOp.STORE,
)

data class RenderPassDescriptor(
    val colorAttachments: List<RenderPassColorAttachment>,
    val depthStencilAttachment: RenderPassDepthStencilAttachment? = null,
    val label: String? = null,
)

fun renderPassColorAttachmentsFromDescribed(
    views: IntArray,
    loadOps: IntArray,
    storeOps: IntArray,
    hasClears: IntArray,
    clearBits: IntArray,
): List<RenderPassColorAttachment> {
    val n = views.size
    val out = ArrayList<RenderPassColorAttachment>(n)
    for (i in 0 until n) {
        if (views[i] == 0) continue
        val base = i * 4
        val clear = if (hasClears.getOrElse(i) { 0 } != 0 && clearBits.size >= base + 4) {
            Color(
                r = Float.fromBits(clearBits[base]).toDouble(),
                g = Float.fromBits(clearBits[base + 1]).toDouble(),
                b = Float.fromBits(clearBits[base + 2]).toDouble(),
                a = Float.fromBits(clearBits[base + 3]).toDouble(),
            )
        } else {
            null
        }
        out.add(
            RenderPassColorAttachment(
                view = GpuHandle(views[i]),
                clearValue = clear,
                loadOp = loadOps.getOrElse(i) { GpuLoadOp.CLEAR },
                storeOp = storeOps.getOrElse(i) { GpuStoreOp.STORE },
            ),
        )
    }
    return out
}
