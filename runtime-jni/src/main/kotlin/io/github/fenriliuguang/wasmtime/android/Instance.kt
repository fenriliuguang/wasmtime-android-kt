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

    /** Call a root export with signature `() -> u32`. */
    fun callUnitToU32(store: Store, exportName: String): Int {
        require(handle != 0L) { "instance closed" }
        require(store.handle != 0L) { "store closed" }
        return NativeBridge.nativeCallUnitToU32(store.handle, handle, exportName)
    }

    /** Call a root export with signature `() -> u64` (bits as signed [Long]). */
    fun callUnitToU64(store: Store, exportName: String): Long {
        require(handle != 0L) { "instance closed" }
        require(store.handle != 0L) { "store closed" }
        return NativeBridge.nativeCallUnitToU64(store.handle, handle, exportName)
    }

    /** Call a root export with signature `(u32, u32) -> u32`. */
    fun callU32U32(store: Store, exportName: String, a: Int, b: Int): Int {
        require(handle != 0L) { "instance closed" }
        require(store.handle != 0L) { "store closed" }
        return NativeBridge.nativeCallU32U32(store.handle, handle, exportName, a, b)
    }

    /**
     * M2: call root export `run: func() -> u32` via Wasmtime `run_concurrent` /
     * `call_concurrent` (host may use `func_wrap_concurrent` + futures).
     *
     * Runs the native pump on a dedicated 8MiB-stack thread. ART instrument
     * threads are ~1MiB; W3 guests that chain two async L2 callbacks plus a
     * sync hop overflow that (`StackOverflowError` on Vivo).
     */
    fun callRunConcurrent(store: Store): Int {
        require(handle != 0L) { "instance closed" }
        require(store.handle != 0L) { "store closed" }
        return onCmPumpThread {
            NativeBridge.nativeCallRunConcurrent(store.handle, handle)
        }
    }

    /**
     * Call a root export with signature `(u64, u32, u32) -> u32`.
     * [a] is treated as unsigned bits (e.g. Android native window handle).
     */
    fun callU64U32U32(store: Store, exportName: String, a: Long, b: Int, c: Int): Int {
        require(handle != 0L) { "instance closed" }
        require(store.handle != 0L) { "store closed" }
        return NativeBridge.nativeCallU64U32U32(store.handle, handle, exportName, a, b, c)
    }

    /**
     * P3-PRIM-3: host-produced `stream<u8>` consumed by guest export `read`.
     * Payload is fixed ASCII `P3ST` (4 bytes). Packed result `(n << 4) | status`.
     */
    fun callStreamRead(store: Store, maxLen: Int = 100): Int {
        require(handle != 0L) { "instance closed" }
        require(store.handle != 0L) { "store closed" }
        require(maxLen > 0) { "maxLen must be positive" }
        return NativeBridge.nativeCallStreamRead(store.handle, handle, maxLen)
    }

    /**
     * P3-PRIM-5: guest `stream.write` (`P3WR`) → host `take` / `StreamConsumer`.
     * Returns consumed byte count (4).
     */
    fun callStreamWrite(store: Store): Int {
        require(handle != 0L) { "instance closed" }
        require(store.handle != 0L) { "store closed" }
        return NativeBridge.nativeCallStreamWrite(store.handle, handle)
    }

    private fun onCmPumpThread(block: () -> Int): Int {
        if (onCmPump.get() == true) {
            return block()
        }
        var result = 0
        var error: Throwable? = null
        val t = Thread(
            null,
            {
                onCmPump.set(true)
                try {
                    result = block()
                } catch (thrown: Throwable) {
                    error = thrown
                }
            },
            "wasmtime-cm-pump",
            CM_PUMP_STACK_BYTES,
        )
        t.start()
        t.join()
        error?.let { throw it }
        return result
    }

    companion object {
        /** Larger than ART's ~1MiB instrument thread (W3 extra JNI hops). */
        private const val CM_PUMP_STACK_BYTES = 8L * 1024L * 1024L
        private val onCmPump = ThreadLocal<Boolean>()
    }
}
