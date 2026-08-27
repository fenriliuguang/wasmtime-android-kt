package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge

/** Component linker. Product [create] omits WebGPU/HTTP fixture constructors. */
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
        /**
         * Product linker. Does **not** export fixture constructors `get-gpu`,
         * `get-device`, `get-gpu-error`, `get-device-lost-info`, or HTTP
         * `[constructor]request` / `[constructor]response`.
         * `[method]gpu.request-adapter` stays. Instruments that import those
         * constructors use [createWithFixtureConstructors].
         */
        fun create(engine: Engine): Linker {
            require(engine.handle != 0L) { "engine closed" }
            return Linker(NativeBridge.nativeLinkerNew(engine.handle))
        }

        /**
         * Test-only linker: same as [create] plus fixture constructors
         * (`get-gpu`, `get-device`, `get-gpu-error`, `get-device-lost-info`,
         * `[constructor]request`, `[constructor]response`).
         * Not product API.
         */
        fun createWithFixtureConstructors(engine: Engine): Linker {
            require(engine.handle != 0L) { "engine closed" }
            return Linker(
                NativeBridge.nativeLinkerNewWithFixtureConstructors(engine.handle),
            )
        }
    }
}
