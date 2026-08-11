package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge

/** Wasmtime store (empty host state for M1). */
class Store private constructor(internal var handle: Long) : AutoCloseable {
    override fun close() {
        val h = handle
        if (h == 0L) return
        handle = 0L
        NativeBridge.nativeStoreClose(h)
    }

    companion object {
        fun create(engine: Engine): Store {
            require(engine.handle != 0L) { "engine closed" }
            return Store(NativeBridge.nativeStoreNew(engine.handle))
        }
    }
}
