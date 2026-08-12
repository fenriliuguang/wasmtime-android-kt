package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import org.junit.Assert.assertNotEquals
import org.junit.Test
import org.junit.runner.RunWith

/**
 * WASI 0.3: `wasi:random/random@0.3.0#get-random-u64` via guest `run`.
 * Two calls must not return the same bits (CSPRNG; not a constant stub).
 */
@RunWith(AndroidJUnit4::class)
class WasiRandomInstrumentedTest {
    @Test
    fun getRandomU64IsNotConstant() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/random_u64.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            val a = instance.callUnitToU64(store, "run")
                            val b = instance.callUnitToU64(store, "run")
                            assertNotEquals(a, b)
                        }
                    }
                }
            }
        }
    }
}
