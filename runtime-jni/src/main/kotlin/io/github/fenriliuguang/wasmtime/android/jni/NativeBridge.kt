package io.github.fenriliuguang.wasmtime.android.jni

/**
 * Minimal JNI surface for M0: prove the cdylib is ours and linked to Wasmtime.
 * Engine / Store / Linker land in M1.
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
}
