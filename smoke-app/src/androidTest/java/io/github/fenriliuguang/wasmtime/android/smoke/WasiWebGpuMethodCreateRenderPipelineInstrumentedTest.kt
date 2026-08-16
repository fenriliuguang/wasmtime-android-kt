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

/** W3+ `[method]gpu-device.create-render-pipeline` via [ExperimentalWebGpuBridge.attachCreateRenderPipeline]. */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodCreateRenderPipelineInstrumentedTest {
    @Test
    fun guestCreateRenderPipelineViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_create_render_pipeline.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachCreateRenderPipeline(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val rep = instance.callRunConcurrent(store)
                                assertNotEquals("render-pipeline rep must be non-zero", 0, rep)
                                assertTrue("render-pipeline rep should be positive", rep > 0)
                            }
                        }
                    }
                }
            }
        }
    }
}
