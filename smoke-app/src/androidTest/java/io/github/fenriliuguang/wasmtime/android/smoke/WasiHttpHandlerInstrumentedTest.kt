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
 * WASI 0.3: `wasi:http` incoming-handler + P010 body `stream<u8>`.
 * `handle` is `async func(own<request>) -> result<own<response>, error-code>` (ok).
 * In-process ABI (not a listening HTTP server). Body: consume-body / response.new.
 */
@RunWith(AndroidJUnit4::class)
class WasiHttpHandlerInstrumentedTest {
    @Test
    fun incomingHandlerRunReturns200() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/http_handler.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(200, instance.callRunConcurrent(store))
                        }
                    }
                }
            }
        }
    }

    @Test
    fun bodyStreamEchoReturnsByteCount() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/http_body.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(4, instance.callRunConcurrent(store))
                        }
                    }
                }
            }
        }
    }
}
