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

/** M1: host resource widget with u32 rep round-trip via guest `run`. */
@RunWith(AndroidJUnit4::class)
class SyncCmResourceInstrumentedTest {
    @Test
    fun guestMakeAndEchoWidgetRep() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("m1/widget_echo.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(42, instance.callU32(store, "run", 42))
                            assertEquals(7, instance.callU32(store, "run", 7))
                        }
                    }
                }
            }
        }
    }
}
