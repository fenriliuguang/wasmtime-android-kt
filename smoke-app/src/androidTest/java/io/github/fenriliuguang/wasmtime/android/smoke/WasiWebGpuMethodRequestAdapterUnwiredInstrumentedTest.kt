package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendKind
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Unwired store: `[method]gpu.request-adapter` returns guest `none` (no trap).
 * Does not instantiate Dawn / mavenLocal host. Same fixture as S2 some-path.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodRequestAdapterUnwiredInstrumentedTest {
    @Test
    fun guestRequestAdapterNoneWhenUnwired() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_request_adapter.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.createWithFixtureConstructors(engine).use { linker ->
                    Store.create(engine).use { store ->
                        assertTrue(store.backendKind is WebGpuBackendKind.None)
                        linker.instantiate(store, component).use { instance ->
                            val harness = instance.callRunConcurrent(store)
                            assertEquals(
                                "unwired request-adapter must be none; guest returns harness 1",
                                1,
                                harness,
                            )
                        }
                    }
                }
            }
        }
    }
}
