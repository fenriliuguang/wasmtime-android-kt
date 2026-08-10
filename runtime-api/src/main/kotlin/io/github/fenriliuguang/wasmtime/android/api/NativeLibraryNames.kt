package io.github.fenriliuguang.wasmtime.android.api

/**
 * Native library basename for [System.loadLibrary].
 * Produces `libwasmtime_android_kt.so` on Android / Linux.
 */
object NativeLibraryNames {
    const val WASMTIME_ANDROID_KT: String = "wasmtime_android_kt"
}
