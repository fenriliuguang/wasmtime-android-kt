package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge

/** Instantiated component. */
class Instance internal constructor(internal var handle: Long) : AutoCloseable {
    override fun close() {
        val h = handle
        if (h == 0L) return
        handle = 0L
        NativeBridge.nativeInstanceClose(h)
    }

    /** Call a root export with signature `(u32) -> u32`. */
    fun callU32(store: Store, exportName: String, arg: Int): Int {
        require(handle != 0L) { "instance closed" }
        require(store.handle != 0L) { "store closed" }
        return NativeBridge.nativeCallU32(store.handle, handle, exportName, arg)
    }
}
