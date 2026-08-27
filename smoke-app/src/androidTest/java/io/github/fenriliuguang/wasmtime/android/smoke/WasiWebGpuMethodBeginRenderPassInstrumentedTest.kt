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
 * L2 `[method]` slice: `get-encoder` + `get-texture-view` then
 * `[method]gpu-command-encoder.begin-render-pass`
 * (color clear 0,0,0,1 + depth-stencil; drops owns;
 * `run` returns 1). Instrument still substitutes Cpu offscreen TextureView via
 * [ExperimentalWebGpuBridge.attachBeginRenderPassClear] + [callRunConcurrent].
 * Flat `command-encoder-begin-render-pass-clear` remains. Not compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodBeginRenderPassInstrumentedTest {
    @Test
    fun guestBeginRenderPassViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_begin_render_pass.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.createWithFixtureConstructors(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachBeginRenderPassClear(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals(
                                    "guest must drop own pass and return harness 1",
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
}
