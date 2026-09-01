package io.github.fenriliuguang.wasmtime.android.jni

import io.github.fenriliuguang.wasmtime.android.internal.ExperimentalHostCallbacks
import io.github.fenriliuguang.wasmtime.android.api.HostU32U32ToU32

/**
 * JNI surface for Track B.
 * M0: identity / version probes.
 * M1: sync CM compile / host import / u32 resource / call export.
 * M2: concurrent async get.
 * M3: experimental request-adapter → Track A L2.
 * M4: flat clear→present experimental host + `(u64,u32,u32)->u32` export.
 * P3: stream.read (host→guest) + stream.write / take (guest→host).
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
    @JvmStatic
    external fun nativeStoreSetExperimentalHost(store: Long, callback: ExperimentalHostCallbacks)
    @JvmStatic external fun nativeStoreSetNativeGpu(store: Long)
    @JvmStatic external fun nativeStorePostGfxVsync(store: Long, frameTimeNanos: Long)
    @JvmStatic external fun nativeStoreCloseGfxOnFrame(store: Long)
    @JvmStatic
    external fun nativeStoreBindCanvasNativeWindow(
        store: Long,
        nativeWindowHandle: Long,
        width: Int,
        height: Int,
    )

    /**
     * H9: `NATIVE_WINDOW_SET_BUFFER_COUNT` before Dawn configure.
     * @return 0 on success; negative if missing/unsupported.
     */
    @JvmStatic
    external fun nativeSetANativeWindowBufferCount(window: Long, count: Int): Int

    @JvmStatic external fun nativeComponentCompile(engine: Long, bytes: ByteArray): Long
    @JvmStatic external fun nativeComponentClose(handle: Long)

    @JvmStatic external fun nativeLinkerNew(engine: Long): Long
    @JvmStatic
    external fun nativeLinkerNewWithFixtureConstructors(engine: Long): Long
    @JvmStatic external fun nativeLinkerClose(handle: Long)

    @JvmStatic external fun nativeInstantiate(linker: Long, store: Long, component: Long): Long
    @JvmStatic external fun nativeInstanceClose(handle: Long)

    /** Call root export `(u32) -> u32`. */
    @JvmStatic
    external fun nativeCallU32(store: Long, instance: Long, exportName: String, arg: Int): Int

    /** Call root export `() -> u32`. */
    @JvmStatic
    external fun nativeCallUnitToU32(store: Long, instance: Long, exportName: String): Int

    /** Call root export `() -> u64` (bits as signed Long). */
    @JvmStatic
    external fun nativeCallUnitToU64(store: Long, instance: Long, exportName: String): Long

    /** Call root export `(u32, u32) -> u32`. */
    @JvmStatic
    external fun nativeCallU32U32(
        store: Long,
        instance: Long,
        exportName: String,
        a: Int,
        b: Int,
    ): Int

    /** M2: `run_concurrent` + `call_concurrent` for export `run: func() -> u32`. */
    @JvmStatic
    external fun nativeCallRunConcurrent(store: Long, instance: Long): Int

    /** M4: call root export `(u64, u32, u32) -> u32` (unsigned window handle as Long bits). */
    @JvmStatic
    external fun nativeCallU64U32U32(
        store: Long,
        instance: Long,
        exportName: String,
        a: Long,
        b: Int,
        c: Int,
    ): Int

    /**
     * P3-PRIM-3: create host `stream<u8>` (`P3ST`) and call guest export
     * `read: func(stream<u8>, u32) -> u32`. Returns Wasmtime packed read result.
     */
    @JvmStatic
    external fun nativeCallStreamRead(store: Long, instance: Long, maxLen: Int): Int

    /**
     * P3-PRIM-5: call guest `run` which `stream.write`s `P3WR` into host `take`
     * (`StreamConsumer` + `future<u32>`). Returns consumed byte count (4).
     */
    @JvmStatic
    external fun nativeCallStreamWrite(store: Long, instance: Long): Int
}
