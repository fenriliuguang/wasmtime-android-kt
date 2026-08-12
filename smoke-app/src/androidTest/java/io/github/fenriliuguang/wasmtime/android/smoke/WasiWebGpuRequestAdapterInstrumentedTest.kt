package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CpuWasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.webgpu.ExperimentalWebGpuBridge
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

/**
 * W1: guest imports transitional flat `wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter`
 * (not `[method]gpu.request-adapter`) → same L2 sync path as M3 via
 * [ExperimentalWebGpuBridge.attachRequestAdapter]. Sync-compat u32 only — not true async (W2).
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuRequestAdapterInstrumentedTest {
    @Test
    fun guestRequestAdapterViaProposalName() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_request_adapter.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachRequestAdapter(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val rep = instance.callUnitToU32(store, "run")
                                assertNotEquals("adapter rep must be non-zero", 0, rep)
                                assertTrue("adapter rep should be positive", rep > 0)
                            }
                        }
                    }
                }
            }
        }
    }
}
