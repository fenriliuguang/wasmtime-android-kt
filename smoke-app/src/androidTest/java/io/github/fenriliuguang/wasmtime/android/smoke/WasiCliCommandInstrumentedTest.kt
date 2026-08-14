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
 * WASI 0.3: command-shaped guest `run: async func() -> u32` (0 = ok).
 * Imports existing `wasi:cli/stdout@0.3.0#write-via-stream`; pumped by `callRunConcurrent`.
 * Not a full command world; no ExperimentalWebGpuBridge.
 */
@RunWith(AndroidJUnit4::class)
class WasiCliCommandInstrumentedTest {
    @Test
    fun commandRunReturnsZero() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/cli_command.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(0, instance.callRunConcurrent(store))
                        }
                    }
                }
            }
        }
    }
}
