package io.github.fenriliuguang.wasi.webgpu.experimental.host

/**
 * Opaque WASI-style resource handle (u32).
 *
 * L2 owns allocation / lookup / drop. L1 must not invent handles.
 */
@JvmInline
value class GpuHandle(val raw: Int) {
    init {
        require(raw != 0) { "handle 0 is reserved as null" }
    }

    companion object {
        val NULL: Int = 0
    }
}

enum class ResourceKind {
    Adapter,
    Device,
    Buffer,
    ShaderModule,
    BindGroupLayout,
    BindGroup,
    PipelineLayout,
    ComputePipeline,
    CommandEncoder,
    ComputePassEncoder,
    CommandBuffer,
    Queue,
    Surface,
    Texture,
    TextureView,
    Sampler,
    RenderPipeline,
    RenderPassEncoder,
}

/**
 * In-memory handle table used by [WasiWebGpuHost] implementations.
 *
 * Thread model: P0 assumes single-threaded host calls (see docs/mapping/threading.md).
 */
class HandleTable {
    private var nextId: Int = 1
    private val entries = LinkedHashMap<Int, Entry>()

    data class Entry(
        val kind: ResourceKind,
        val resource: Any,
    )

    fun <T : Any> insert(kind: ResourceKind, resource: T): GpuHandle {
        val id = nextId++
        if (nextId == 0) nextId = 1
        entries[id] = Entry(kind, resource)
        return GpuHandle(id)
    }

    fun contains(handle: GpuHandle): Boolean = entries.containsKey(handle.raw)

    @Suppress("UNCHECKED_CAST")
    fun <T : Any> get(handle: GpuHandle, kind: ResourceKind): T {
        val entry = entries[handle.raw]
            ?: throw HostException.InvalidHandle(handle, "unknown handle")
        if (entry.kind != kind) {
            throw HostException.InvalidHandle(
                handle,
                "expected $kind but found ${entry.kind}",
            )
        }
        return entry.resource as T
    }

    fun drop(handle: GpuHandle): Entry {
        return tryDrop(handle)
            ?: throw HostException.InvalidHandle(handle, "already dropped or unknown")
    }

    /** Remove [handle] if present; returns null when already dropped / unknown. */
    fun tryDrop(handle: GpuHandle): Entry? = entries.remove(handle.raw)

    /** Snapshot of live handles with [kind] (safe to drop while iterating the returned list). */
    fun handlesOfKind(kind: ResourceKind): List<GpuHandle> =
        entries.mapNotNull { (id, entry) ->
            if (entry.kind == kind) GpuHandle(id) else null
        }

    fun size(): Int = entries.size

    fun clear() {
        entries.clear()
    }
}
