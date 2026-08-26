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
 * WASI 0.3: `wasi:clocks/system-clock@0.3.0#now` via guest `run`.
 * Official `instant` record `{seconds: s64, nanoseconds: u32}`; guest returns seconds.
 */
@RunWith(AndroidJUnit4::class)
class WasiSystemClockInstrumentedTest {
    @Test
    fun systemNowIsPlausibleUnixSeconds() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/system_now.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            val secs = instance.callUnitToU64(store, "run")
                            assertTrue(
                                "system-clock seconds out of expected unix range: $secs",
                                secs > 1_704_067_200L && secs < 4_102_444_800L,
                            )
                        }
                    }
                }
            }
        }
    }
}
