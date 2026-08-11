package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

/**
 * P3-PRIM-3: host [StreamReader] with fixed `P3ST` bytes; guest `stream.read`.
 * Packed result `(4 << 4) | 1` = 65.
 */
@RunWith(AndroidJUnit4::class)
class StreamReadInstrumentedTest {
    @Test
    fun guestReadsHostProducedStream() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("p3/stream_read.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(65, instance.callStreamRead(store, maxLen = 100))
                        }
                    }
                }
            }
        }
    }
}
