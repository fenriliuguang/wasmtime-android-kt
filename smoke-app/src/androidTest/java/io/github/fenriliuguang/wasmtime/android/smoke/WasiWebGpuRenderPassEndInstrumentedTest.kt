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
 * `wasi:webgpu/webgpu@0.3.0-rc.2#render-pass-end` (not
 * `[method]gpu-render-pass-encoder.end`) after adapter → device → encoder →
 * begin-clear, via [ExperimentalWebGpuBridge.attachRenderPassEnd] +
 * [callRunConcurrent]. Guest passes stub view 23; the attach path substitutes a
 * Cpu offscreen TextureView (not experimental surface / present). Experimental
 * flat sync path remains separate. Not full wasi:webgpu compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuRenderPassEndInstrumentedTest {
    @Test
    fun guestRenderPassEndViaProposalNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_render_pass_end.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.createWithFixtureConstructors(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachRenderPassEnd(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val rep = instance.callRunConcurrent(store)
                                assertNotEquals("render-pass rep must be non-zero", 0, rep)
                                assertTrue("render-pass rep should be positive", rep > 0)
                            }
                        }
                    }
                }
            }
        }
    }
}
