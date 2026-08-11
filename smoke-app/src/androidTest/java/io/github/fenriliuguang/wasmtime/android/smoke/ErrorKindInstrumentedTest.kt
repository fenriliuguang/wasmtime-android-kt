package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.api.WasmtimeCompileException
import io.github.fenriliuguang.wasmtime.android.api.WasmtimeException
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test
import org.junit.runner.RunWith

/** M5: JNI throws typed [WasmtimeException] subclasses. */
@RunWith(AndroidJUnit4::class)
class ErrorKindInstrumentedTest {
    @Test
    fun invalidComponentBytesThrowCompileException() {
        Engine.create().use { engine ->
            try {
                Component.compile(engine, byteArrayOf(0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00))
                fail("expected WasmtimeCompileException")
            } catch (e: WasmtimeCompileException) {
                assertEquals(WasmtimeException.Kind.COMPILE, e.kind)
                assertTrue(e.message?.isNotBlank() == true)
            }
        }
    }
}
