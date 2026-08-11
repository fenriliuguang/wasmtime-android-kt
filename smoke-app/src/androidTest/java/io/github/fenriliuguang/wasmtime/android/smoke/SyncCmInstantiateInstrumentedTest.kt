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

/** M1 slice 1: compile + instantiate + call u32 export (no host imports). */
@RunWith(AndroidJUnit4::class)
class SyncCmInstantiateInstrumentedTest {
    @Test
    fun compileInstantiateAndCallAddOne() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("m1/add_one.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(3, instance.callU32(store, "run", 2))
                            assertEquals(1, instance.callU32(store, "run", 0))
                        }
                    }
                }
            }
        }
    }
}
