package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.api.HostU32Supplier
import io.github.fenriliuguang.wasmtime.android.api.HostU32U32ToU32
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackend
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendFactory
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendKind
import io.github.fenriliuguang.wasmtime.android.internal.ExperimentalHostCallbacks
import io.github.fenriliuguang.wasmtime.android.internal.WebGpuBackendHostAttach
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

    /**
     * **Not product API.** Instrument / leftover host-import tests only.
     * Apps use [setWebGpuBackend].
     */
    @Deprecated("Not product API. Use setWebGpuBackend.")
    fun setHostAdd(callback: HostU32U32ToU32) {
        require(handle != 0L) { "store closed" }
        NativeBridge.nativeStoreSetHostAdd(handle, callback)
    }

    /**
     * **Not product API.** `:host-dawn` instruments may still attach a partial
     * experimental table. Apps use [setWebGpuBackend].
     */
    @Deprecated("Not product API. Use setWebGpuBackend.")
    fun setExperimentalHost(callback: ExperimentalHostCallbacks) {
        require(handle != 0L) { "store closed" }
        NativeBridge.nativeStoreSetExperimentalHost(handle, callback)
        if (backendKind is WebGpuBackendKind.None) {
            backendKind = WebGpuBackendKind.Custom("experimental-host")
        }
    }

    /**
     * Attach a [WebGpuBackend] (explicit, preferred). Replaces any previous
     * experimental host callbacks **and** any ServiceLoader attach. Always
     * wins over [discoverWebGpuBackend] / [createWithDiscoveredBackend].
     * The backend must implement the internal JNI attach
     * (`GpuBackends.dawn()` / `cpu()`).
     */
    fun setWebGpuBackend(backend: WebGpuBackend) {
        require(handle != 0L) { "store closed" }
        val attach =
            backend as? WebGpuBackendHostAttach
                ?: throw IllegalArgumentException(
                    "WebGpuBackend must implement WebGpuBackendHostAttach (use GpuBackends.dawn())",
                )
        attachedBackend?.close()
        attachedBackend = backend
        backendKind = kindFor(backend.id)
        NativeBridge.nativeStoreSetExperimentalHost(handle, attach.attachExperimentalHost())
    }

    /**
     * Post one Choreographer vsync beat into `wasi-gfx` `surface.on-frame`.
     * 1-slot: drops the beat if the previous event is still unconsumed.
     * Display rate is not capped here: pin `frame-event` has no timestamp
     * ([frameTimeNanos] is accepted for call-site alignment with `doFrame`
     * but is not a rAF delta). Guest motion delta is `wasi:clocks`
     * `monotonic-clock.now`. The CM stream write runs on GpuThread.
     */
    @JvmOverloads
    fun postGfxVsync(@Suppress("UNUSED_PARAMETER") frameTimeNanos: Long = 0L) {
        require(handle != 0L) { "store closed" }
        NativeBridge.nativeStorePostGfxVsync(handle)
    }

    /**
     * Close the `on-frame` stream (`surfaceDestroyed`). Unblocks guest `run`.
     */
    fun closeGfxOnFrame() {
        require(handle != 0L) { "store closed" }
        NativeBridge.nativeStoreCloseGfxOnFrame(handle)
    }

    /**
     * **Not product API.** Convenience for leftover `request-adapter` → `u32` smokes.
     */
    @Deprecated("Not product API. Use setWebGpuBackend.")
    fun setRequestAdapter(callback: HostU32Supplier) {
        @Suppress("DEPRECATION")
        setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = callback.invoke()
            },
        )
    }

    companion object {
        private val logger = Logger.getLogger(Store::class.java.name)

        /**
         * Product default: **no** ServiceLoader. Apps that want the default
         * `android-webgpu` bundle convenience use [createWithDiscoveredBackend].
         *
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

        /**
         * Default-bundle convenience: ServiceLoader [WebGpuBackendFactory]
         * (prefer `id == "dawn"`). Zero factories leave the store unwired
         * (`request-adapter` **`none`**). [setWebGpuBackend] always wins over
         * this path (call it after create, or skip this factory).
         */
        @JvmStatic
        fun createWithDiscoveredBackend(engine: Engine): Store =
            create(engine, discoverWebGpuBackend = true)

        private fun kindFor(id: String): WebGpuBackendKind =
            when (id) {
                "dawn" -> WebGpuBackendKind.Dawn
                "none" -> WebGpuBackendKind.None
                else -> WebGpuBackendKind.Custom(id)
            }

        /**
         * Pick among ServiceLoader factories. Empty → `null` (unwired /
         * `request-adapter` none). Several: prefer `id == "dawn"`.
         */
        internal fun pickDiscoveredBackend(factories: List<WebGpuBackendFactory>): WebGpuBackend? {
            if (factories.isEmpty()) {
                return null
            }
            if (factories.size == 1) {
                return factories[0].create()
            }
            val created = factories.map { it.create() }
            val dawn = created.firstOrNull { it.id == "dawn" }
            if (dawn != null) {
                created.filter { it !== dawn }.forEach { runCatching { it.close() } }
                return dawn
            }
            logger.warning(
                "Multiple WebGpuBackendFactory entries (${created.map { it.id }}); using ${created[0].id}",
            )
            created.drop(1).forEach { runCatching { it.close() } }
            return created[0]
        }
    }

    /**
     * ServiceLoader attach. No-op when a backend is already attached
     * ([setWebGpuBackend] always wins) or when no factory is on the classpath
     * (unwired `request-adapter` `none`). Does not instantiate Dawn unless a
     * factory is present.
     */
    fun discoverWebGpuBackend() {
        require(handle != 0L) { "store closed" }
        if (attachedBackend != null) {
            return
        }
        val picked =
            pickDiscoveredBackend(ServiceLoader.load(WebGpuBackendFactory::class.java).toList())
                ?: return
        setWebGpuBackend(picked)
    }
}
