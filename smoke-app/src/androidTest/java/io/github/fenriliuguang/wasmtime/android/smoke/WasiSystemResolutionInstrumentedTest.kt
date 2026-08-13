package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

/**
 * WASI 0.3: `wasi:clocks/system-clock@0.3.0#resolution` via guest `run`.
 * Transitional `func() -> u64` nanoseconds (official WIT may be a datetime record);
 * production host returns 1 ns. Timezone is not in this slice.
 */
@RunWith(AndroidJUnit4::class)
class WasiSystemResolutionInstrumentedTest {
    @Test
    fun systemResolutionIsOneNanosecond() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/system_resolution.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            val result = instance.callUnitToU64(store, "run")
                            assertTrue("resolution must be 1 ns: got $result", result == 1L)
                        }
                    }
                }
            }
        }
    }
}
