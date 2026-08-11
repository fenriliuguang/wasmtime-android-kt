package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge

/** Component linker (no host imports in M1 slice 1). */
class Linker private constructor(internal var handle: Long) : AutoCloseable {
    override fun close() {
        val h = handle
        if (h == 0L) return
        handle = 0L
        NativeBridge.nativeLinkerClose(h)
    }

    fun instantiate(store: Store, component: Component): Instance {
        require(handle != 0L) { "linker closed" }
        require(store.handle != 0L) { "store closed" }
        require(component.handle != 0L) { "component closed" }
        val h = NativeBridge.nativeInstantiate(handle, store.handle, component.handle)
        check(h != 0L) { "instantiate returned null handle" }
        return Instance(h)
    }

    companion object {
        fun create(engine: Engine): Linker {
            require(engine.handle != 0L) { "engine closed" }
            return Linker(NativeBridge.nativeLinkerNew(engine.handle))
        }
    }
}
