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
 * W1: guest writes three 4-byte chunks (`P3C1P3C2P3C3`); host `take-chunks`
 * pipes a 2-byte-per-poll [StreamConsumer] (backpressure). Guest `run` returns
 * 12. Reuses `Instance.callStreamWrite` (same 8MiB CM pump, export `run`).
 */
@RunWith(AndroidJUnit4::class)
class StreamChunksInstrumentedTest {
    @Test
    fun hostConsumesMultiChunkGuestStreamWithBackpressure() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("p3/stream_chunks.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(12, instance.callStreamWrite(store))
                        }
                    }
                }
            }
        }
    }
}
