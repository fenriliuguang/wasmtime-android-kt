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
 * W3 `[method]` slice: guest imports `get-device` then
 * `wasi:webgpu/webgpu@0.3.0-rc.2#[method]gpu-device.create-command-encoder`
 * (sync; resource self; still u32, no descriptor) via
 * [ExperimentalWebGpuBridge.attachCreateCommandEncoder] + [callRunConcurrent].
 * Flat `device-create-command-encoder` remains registered. Not full wasi:webgpu
 * compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodCreateCommandEncoderInstrumentedTest {
    @Test
    fun guestCreateCommandEncoderViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_create_command_encoder.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
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
