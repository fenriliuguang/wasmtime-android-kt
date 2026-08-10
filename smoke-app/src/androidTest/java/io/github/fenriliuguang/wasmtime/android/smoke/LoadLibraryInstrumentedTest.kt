package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge
import io.github.fenriliuguang.wasmtime.android.jni.NativeLoader
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

/** M0 DoD: ART loadLibrary + JNI_OnLoad must not crash. */
@RunWith(AndroidJUnit4::class)
class LoadLibraryInstrumentedTest {
    @Test
    fun loadLibraryAndQueryWasmtimeVersion() {
        NativeLoader.ensureLoaded()
        val id = NativeBridge.nativeRuntimeId()
        val ver = NativeBridge.nativeWasmtimeVersion()
        assertTrue(id.contains("wasmtime-android-kt"))
        assertFalse(ver.isBlank())
    }
}
