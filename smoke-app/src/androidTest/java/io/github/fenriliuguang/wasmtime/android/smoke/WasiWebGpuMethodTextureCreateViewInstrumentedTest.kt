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
 * L2 `[method]` slice: guest imports `get-texture` then
 * `[method]gpu-texture.create-view`
 * (`option<gpu-texture-view-descriptor>` = some(rgba8unorm + d2 + aspect all
 * + mip 0/1 + array 0/1)
 * → `own<gpu-texture-view>`; drops the own; `run` returns 1) via
 * [ExperimentalWebGpuBridge.attachCreateTextureView] + [callRunConcurrent].
 * Flat names remain registered. Not compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodTextureCreateViewInstrumentedTest {
    @Test
    fun guestCreateTextureViewViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_texture_create_view.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachCreateTextureView(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals(
                                    "guest must drop owns and return harness 1",
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
