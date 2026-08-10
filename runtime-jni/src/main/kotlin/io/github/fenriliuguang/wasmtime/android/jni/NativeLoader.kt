package io.github.fenriliuguang.wasmtime.android.jni

import io.github.fenriliuguang.wasmtime.android.api.NativeLibraryNames

/**
 * Loads [libwasmtime_android_kt] once per process.
 * Call before any other JNI entry (M1+).
 */
object NativeLoader {
    @Volatile
    private var loaded: Boolean = false

    fun ensureLoaded() {
        if (loaded) return
        synchronized(this) {
            if (loaded) return
            System.loadLibrary(NativeLibraryNames.WASMTIME_ANDROID_KT)
            loaded = true
        }
    }
}
