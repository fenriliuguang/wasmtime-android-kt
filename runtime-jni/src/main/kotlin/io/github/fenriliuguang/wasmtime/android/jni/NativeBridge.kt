package io.github.fenriliuguang.wasmtime.android.jni

import io.github.fenriliuguang.wasmtime.android.api.HostU32U32ToU32

/**
 * JNI surface for Track B.
 * M0: identity / version probes.
 * M1: sync CM compile / host import / u32 resource / call export.
 */
object NativeBridge {
    init {
        NativeLoader.ensureLoaded()
    }

    /** Crate / binder identity string (UTF-8). */
    @JvmStatic
    external fun nativeRuntimeId(): String

    /** Upstream Wasmtime crate version string (UTF-8). */
    @JvmStatic
    external fun nativeWasmtimeVersion(): String

    @JvmStatic external fun nativeEngineNew(): Long
    @JvmStatic external fun nativeEngineClose(handle: Long)

    @JvmStatic external fun nativeStoreNew(engine: Long): Long
    @JvmStatic external fun nativeStoreClose(handle: Long)
    @JvmStatic external fun nativeStoreSetHostAdd(store: Long, callback: HostU32U32ToU32)

    @JvmStatic external fun nativeComponentCompile(engine: Long, bytes: ByteArray): Long
    @JvmStatic external fun nativeComponentClose(handle: Long)

    @JvmStatic external fun nativeLinkerNew(engine: Long): Long
    @JvmStatic external fun nativeLinkerClose(handle: Long)

    @JvmStatic external fun nativeInstantiate(linker: Long, store: Long, component: Long): Long
    @JvmStatic external fun nativeInstanceClose(handle: Long)

    /** Call root export `(u32) -> u32`. */
    @JvmStatic
    external fun nativeCallU32(store: Long, instance: Long, exportName: String, arg: Int): Int

    /** Call root export `(u32, u32) -> u32`. */
    @JvmStatic
    external fun nativeCallU32U32(
        store: Long,
        instance: Long,
        exportName: String,
        a: Int,
        b: Int,
    ): Int
}
