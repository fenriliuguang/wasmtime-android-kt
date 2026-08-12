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
 * P3-PRIM-5: guest writes `P3WR` via `stream.write`; host `take` pipes
 * [StreamConsumer] and returns byte count 4 via `future<u32>`.
 */
@RunWith(AndroidJUnit4::class)
class StreamWriteInstrumentedTest {
    @Test
    fun hostConsumesGuestWrittenStream() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("p3/stream_write.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(4, instance.callStreamWrite(store))
                        }
                    }
                }
            }
        }
    }
}
