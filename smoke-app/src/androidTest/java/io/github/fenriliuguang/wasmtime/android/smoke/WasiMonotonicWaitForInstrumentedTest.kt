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
 * WASI 0.3: `wasi:clocks/monotonic-clock@0.3.0#wait-for` via guest async `run`.
 * Driven by the same `callRunConcurrent` / `run_concurrent` path as M2.
 */
@RunWith(AndroidJUnit4::class)
class WasiMonotonicWaitForInstrumentedTest {
    @Test
    fun waitForCompletesAndReturnsOne() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/monotonic_wait_for.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(1, instance.callRunConcurrent(store))
                        }
                    }
                }
            }
        }
    }
}
