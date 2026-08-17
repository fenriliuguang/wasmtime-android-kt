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
 * S8 `[method]` slice: guest imports `get-encoder` then
 * `[method]gpu-command-encoder.begin-compute-pass`
 * (`option<gpu-compute-pass-descriptor>` = none → `own<gpu-compute-pass-encoder>`;
 * drops the own; `run` returns 1) via
 * [ExperimentalWebGpuBridge.attachBeginComputePass] + [callRunConcurrent].
 * Flat names remain registered. Not compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodBeginComputePassInstrumentedTest {
    @Test
    fun guestBeginComputePassViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_begin_compute_pass.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachBeginComputePass(store, host)
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
