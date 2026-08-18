package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CpuWasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.webgpu.ExperimentalWebGpuBridge
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

/**
 * S6+ `[method]` slice: `get-pass` + `get-render-pipeline` then
 * `[method]gpu-render-pass-encoder.set-pipeline` (borrow pipeline;
 * L2 still host-fixed triangle pipeline + offscreen view) via
 * [ExperimentalWebGpuBridge.attachRenderPassSetPipeline] + [callRunConcurrent].
 * Not compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodRenderPassSetPipelineInstrumentedTest {
    @Test
    fun guestRenderPassSetPipelineViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_render_pass_set_pipeline.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachRenderPassSetPipeline(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals("guest must return harness 1 after set-pipeline", 1, harness)
                            }
                        }
                    }
                }
            }
        }
    }
}
