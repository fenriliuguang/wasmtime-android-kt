package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge
import io.github.fenriliuguang.wasmtime.android.jni.NativeLoader

/** Upstream Wasmtime engine (M1). Close before process exit when possible. */
class Engine private constructor(internal var handle: Long) : AutoCloseable {
    override fun close() {
        val h = handle
        if (h == 0L) return
        handle = 0L
        NativeBridge.nativeEngineClose(h)
    }

    companion object {
        fun create(): Engine {
            NativeLoader.ensureLoaded()
            return Engine(NativeBridge.nativeEngineNew())
        }
    }
}
