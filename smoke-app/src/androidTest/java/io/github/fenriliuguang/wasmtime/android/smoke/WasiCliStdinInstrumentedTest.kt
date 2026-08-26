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
 * WASI 0.3: `wasi:cli/stdin@0.3.0#read-via-stream` via guest `run`.
 * Official `tuple<stream<u8>, future<result<_, error-code>>>`; payload `IN\n` (3 bytes).
 */
@RunWith(AndroidJUnit4::class)
class WasiCliStdinInstrumentedTest {
    @Test
    fun readViaStreamReturnsByteCount() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/cli_stdin.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(3, instance.callStreamWrite(store))
                        }
                    }
                }
            }
        }
    }
}
