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

/** M1: guest export calls sync host import `add` implemented in Kotlin. */
@RunWith(AndroidJUnit4::class)
class SyncCmHostImportInstrumentedTest {
    @Test
    fun guestRunCallsKotlinHostAdd() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("m1/host_add.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        @Suppress("DEPRECATION")
                        store.setHostAdd { a, b -> a + b + 10 }
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(15, instance.callU32U32(store, "run", 2, 3))
                        }
                    }
                }
            }
        }
    }
}
