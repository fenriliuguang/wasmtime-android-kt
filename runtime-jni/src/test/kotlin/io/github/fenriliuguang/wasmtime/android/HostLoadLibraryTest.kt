package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge
import io.github.fenriliuguang.wasmtime.android.jni.NativeLoader
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Test
import java.io.File

/**
 * Optional desktop shell smoke (M5).
 * Requires [scripts/build-native-host.ps1] → `desktop/jniLibs/` and
 * `java.library.path` (configured in `runtime-jni/build.gradle.kts`).
 */
class HostLoadLibraryTest {
    @Test
    fun loadLibraryAndQueryWasmtimeVersion() {
        val jniDir = File(System.getProperty("wasmtime.desktop.jniLibs", ""))
        assumeTrue(
            "Run .\\scripts\\build-native-host.ps1 first (desktop/jniLibs missing or empty)",
            jniDir.isDirectory &&
                (
                    File(jniDir, "wasmtime_android_kt.dll").isFile ||
                        File(jniDir, "libwasmtime_android_kt.so").isFile ||
                        File(jniDir, "libwasmtime_android_kt.dylib").isFile
                ),
        )
        NativeLoader.ensureLoaded()
        val id = NativeBridge.nativeRuntimeId()
        val ver = NativeBridge.nativeWasmtimeVersion()
        assertTrue(id.contains("wasmtime-android-kt"))
        assertFalse(ver.isBlank())
    }
}
