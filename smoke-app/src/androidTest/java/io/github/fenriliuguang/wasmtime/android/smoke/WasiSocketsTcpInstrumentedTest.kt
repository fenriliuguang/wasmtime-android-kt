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
 * WASI 0.3: `wasi:sockets` TCP loopback echo via guest async `run`.
 * Requires `INTERNET` (including loopback). Host IO is on a helper thread;
 * Guest `create-tcp-socket(ipv4)` then `connect(ipv4 loopback)` + stream echo `P3SK`.
 */
@RunWith(AndroidJUnit4::class)
class WasiSocketsTcpInstrumentedTest {
    @Test
    fun loopbackEchoReturnsByteCount() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/sockets_tcp.wasm")
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
