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
 * WASI 0.3: `wasi:cli/stdout@0.3.0#write-via-stream` via guest `run`.
 * Transitional: host returns `future<u32>` byte count (4 for `OUT\n`).
 */
@RunWith(AndroidJUnit4::class)
class WasiCliStdoutInstrumentedTest {
    @Test
    fun writeViaStreamReturnsByteCount() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/cli_stdout.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            // Same pump as P3 stream write: call_async on export `run`.
                            assertEquals(4, instance.callStreamWrite(store))
                        }
                    }
                }
            }
        }
    }
}
