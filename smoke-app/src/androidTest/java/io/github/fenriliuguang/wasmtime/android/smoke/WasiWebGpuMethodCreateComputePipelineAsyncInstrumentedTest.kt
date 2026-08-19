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
 * S6+ `[method]` slice: guest imports `get-device` + `get-shader-module` then
 * `[method]gpu-device.create-compute-pipeline-async`
 * (async result own pipeline / create-pipeline-error; shader borrow, entry-point=`main`, layout auto, label=`l2`;
 * drops the own on ok; `run` returns 1). L2 described shader handle + entry-point + layout + label.
 * via [ExperimentalWebGpuBridge.attachCreateComputePipelineAsync] + [callRunConcurrent].
 * Not full wasi:webgpu compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodCreateComputePipelineAsyncInstrumentedTest {
    @Test
    fun guestCreateComputePipelineAsyncViaMethodNameAsync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_create_compute_pipeline_async.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachCreateComputePipelineAsync(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals(
                                    "guest must drop own<gpu-compute-pipeline> on ok and return harness 1",
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
