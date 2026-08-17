package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.api.ExperimentalHostCallbacks
import io.github.fenriliuguang.wasmtime.android.api.HostU32Supplier
import io.github.fenriliuguang.wasmtime.android.api.HostU32U32ToU32
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackend
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendFactory
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendKind
import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge
import java.util.ServiceLoader
import java.util.logging.Logger

/** Wasmtime store (host callbacks + resource table). */
class Store private constructor(internal var handle: Long) : AutoCloseable {
    @Volatile
    var backendKind: WebGpuBackendKind = WebGpuBackendKind.None
        private set

    private var attachedBackend: WebGpuBackend? = null

    override fun close() {
        val backend = attachedBackend
        attachedBackend = null
        backend?.close()
        val h = handle
        if (h == 0L) return
        handle = 0L
        NativeBridge.nativeStoreClose(h)
    }

    /** Register Kotlin implementation for root import `add: func(u32,u32)->u32`. */
    fun setHostAdd(callback: HostU32U32ToU32) {
        require(handle != 0L) { "store closed" }
        NativeBridge.nativeStoreSetHostAdd(handle, callback)
    }

    /**
     * Register Kotlin [ExperimentalHostCallbacks] for leftover flat u32-rep
     * imports. Prefer [setWebGpuBackend] for product code.
     */
    fun setExperimentalHost(callback: ExperimentalHostCallbacks) {
        require(handle != 0L) { "store closed" }
        NativeBridge.nativeStoreSetExperimentalHost(handle, callback)
        if (backendKind is WebGpuBackendKind.None) {
            backendKind = WebGpuBackendKind.Custom("experimental-host")
        }
    }

    /**
     * Attach a [WebGpuBackend] (explicit, preferred). Replaces any previous
     * experimental host callbacks.
     */
    fun setWebGpuBackend(backend: WebGpuBackend) {
        require(handle != 0L) { "store closed" }
        attachedBackend?.close()
        attachedBackend = backend
        backendKind = kindFor(backend.id)
        NativeBridge.nativeStoreSetExperimentalHost(handle, backend.hostCallbacks())
    }

    /**
     * Convenience for M3: only `request-adapter` → `u32` rep.
     */
    fun setRequestAdapter(callback: HostU32Supplier) {
        setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = callback.invoke()
            },
        )
    }

    companion object {
        private val logger = Logger.getLogger(Store::class.java.name)

        /**
         * @param discoverWebGpuBackend if true and no explicit backend is set,
         *   load [WebGpuBackendFactory] via [ServiceLoader]. Zero factories →
         *   unwired (`request-adapter` `none`). Several: prefer `id == "dawn"`.
         */
        @JvmOverloads
        fun create(engine: Engine, discoverWebGpuBackend: Boolean = false): Store {
            require(engine.handle != 0L) { "engine closed" }
            val store = Store(NativeBridge.nativeStoreNew(engine.handle))
            if (discoverWebGpuBackend) {
                store.discoverWebGpuBackend()
            }
            return store
        }

        private fun kindFor(id: String): WebGpuBackendKind =
            when (id) {
                "dawn" -> WebGpuBackendKind.Dawn
                "none" -> WebGpuBackendKind.None
                else -> WebGpuBackendKind.Custom(id)
            }
    }

    /**
     * ServiceLoader attach. No-op when no factory is on the classpath.
     * Does not instantiate Dawn unless a factory is present.
     */
    fun discoverWebGpuBackend() {
        require(handle != 0L) { "store closed" }
        val factories = ServiceLoader.load(WebGpuBackendFactory::class.java).toList()
        if (factories.isEmpty()) {
            return
        }
        val factory =
            if (factories.size == 1) {
                factories[0]
            } else {
                val created = factories.map { it.create() }
                val dawn = created.firstOrNull { it.id == "dawn" }
                if (dawn != null) {
                    created.filter { it !== dawn }.forEach { runCatching { it.close() } }
                    setWebGpuBackend(dawn)
                    return
                }
                logger.warning(
                    "Multiple WebGpuBackendFactory entries (${created.map { it.id }}); using ${created[0].id}",
                )
                created.drop(1).forEach { runCatching { it.close() } }
                setWebGpuBackend(created[0])
                return
            }
        setWebGpuBackend(factory.create())
    }
}
