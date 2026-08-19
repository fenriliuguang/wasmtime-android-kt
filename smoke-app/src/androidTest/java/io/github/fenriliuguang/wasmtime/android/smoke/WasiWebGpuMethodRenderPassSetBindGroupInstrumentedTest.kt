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
 * L2 `[method]` slice: `get-pass` + `get-bind-group` then
 * `[method]gpu-render-pass-encoder.set-bind-group` (index 0, some bind-group,
 * offsets none → result ok; described JNI + offscreen view) via
 * [ExperimentalWebGpuBridge.attachRenderPassSetBindGroup] + [callRunConcurrent].
 * Not compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodRenderPassSetBindGroupInstrumentedTest {
    @Test
    fun guestRenderPassSetBindGroupViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_render_pass_set_bind_group.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachRenderPassSetBindGroup(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals(
                                    "guest must return harness 1 after set-bind-group ok",
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
