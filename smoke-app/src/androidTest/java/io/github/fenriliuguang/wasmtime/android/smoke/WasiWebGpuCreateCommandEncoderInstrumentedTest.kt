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
 * W3 transitional flat sync: guest imports
 * `wasi:webgpu/webgpu@0.3.0-rc.2#device-create-command-encoder` (not
 * `[method]gpu-device.create-command-encoder`) after adapter → device, via
 * [ExperimentalWebGpuBridge.attachCreateCommandEncoder] + [callRunConcurrent]
 * (export is async because it calls async adapter/device). Experimental
 * flat sync path remains separate. Not full wasi:webgpu compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuCreateCommandEncoderInstrumentedTest {
    @Test
    fun guestCreateCommandEncoderViaProposalNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_create_command_encoder.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.createWithFixtureConstructors(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachCreateCommandEncoder(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val rep = instance.callRunConcurrent(store)
                                assertNotEquals("encoder rep must be non-zero", 0, rep)
                                assertTrue("encoder rep should be positive", rep > 0)
                            }
                        }
                    }
                }
            }
        }
    }
}
