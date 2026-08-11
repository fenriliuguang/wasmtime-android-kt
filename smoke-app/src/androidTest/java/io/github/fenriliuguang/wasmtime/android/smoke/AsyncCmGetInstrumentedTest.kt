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
 * M2 hard gate: guest `run` sync-lowers async host `get`, hosted by
 * `func_wrap_concurrent` + `FutureReader` complete, driven by `run_concurrent`.
 */
@RunWith(AndroidJUnit4::class)
class AsyncCmGetInstrumentedTest {
    @Test
    fun guestObservesCompletedHostFuture() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("m2/async_get.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(42, instance.callRunConcurrent(store))
                        }
                    }
                }
            }
        }
    }
}
