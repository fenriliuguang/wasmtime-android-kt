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
 * S6+ `[method]` slice: `get-pass` + `get-buffer` then
 * `[method]gpu-render-pass-encoder.set-vertex-buffer` (slot 0, some buffer,
 * offset/size none; L2 still host-fixed VERTEX slot 0 + offscreen view) via
 * [ExperimentalWebGpuBridge.attachRenderPassSetVertexBuffer] + [callRunConcurrent].
 * Not compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodRenderPassSetVertexBufferInstrumentedTest {
    @Test
    fun guestRenderPassSetVertexBufferViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_render_pass_set_vertex_buffer.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachRenderPassSetVertexBuffer(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals(
                                    "guest must return harness 1 after set-vertex-buffer",
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
