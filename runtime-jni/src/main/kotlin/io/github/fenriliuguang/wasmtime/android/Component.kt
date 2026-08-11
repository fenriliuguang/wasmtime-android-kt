package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge

/** Compiled Component Model module. */
class Component private constructor(internal var handle: Long) : AutoCloseable {
    override fun close() {
        val h = handle
        if (h == 0L) return
        handle = 0L
        NativeBridge.nativeComponentClose(h)
    }

    companion object {
        fun compile(engine: Engine, bytes: ByteArray): Component {
            require(engine.handle != 0L) { "engine closed" }
            val h = NativeBridge.nativeComponentCompile(engine.handle, bytes)
            check(h != 0L) { "component compile returned null handle" }
            return Component(h)
        }
    }
}
