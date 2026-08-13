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
 * W2 remainder: guest imports transitional flat
 * `wasi:webgpu/webgpu@0.3.0-rc.2#adapter-request-device` (async; not
 * `[method]gpu-adapter.request-device`) after `request-adapter`, via
 * [ExperimentalWebGpuBridge.attachRequestDevice] + [callRunConcurrent].
 * Experimental flat sync path remains separate. Not full wasi:webgpu compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuRequestDeviceInstrumentedTest {
    @Test
    fun guestRequestDeviceViaProposalNameAsync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_request_device.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachRequestDevice(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val rep = instance.callRunConcurrent(store)
                                assertNotEquals("device rep must be non-zero", 0, rep)
                                assertTrue("device rep should be positive", rep > 0)
                            }
                        }
                    }
                }
            }
        }
    }
}
