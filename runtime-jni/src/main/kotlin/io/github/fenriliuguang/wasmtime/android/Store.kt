package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.api.HostU32Supplier
import io.github.fenriliuguang.wasmtime.android.api.HostU32U32ToU32
import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge

/** Wasmtime store (host callbacks + resource table). */
class Store private constructor(internal var handle: Long) : AutoCloseable {
    override fun close() {
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
     * Register Kotlin implementation for
     * `experimental:webgpu-cm/host@0.8.0#request-adapter` → `u32` rep.
     */
    fun setRequestAdapter(callback: HostU32Supplier) {
        require(handle != 0L) { "store closed" }
        NativeBridge.nativeStoreSetRequestAdapter(handle, callback)
    }

    companion object {
        fun create(engine: Engine): Store {
            require(engine.handle != 0L) { "engine closed" }
            return Store(NativeBridge.nativeStoreNew(engine.handle))
        }
    }
}
