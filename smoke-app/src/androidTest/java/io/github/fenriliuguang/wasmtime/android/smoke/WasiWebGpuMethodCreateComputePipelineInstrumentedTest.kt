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
 * `wasi:webgpu/webgpu@0.3.0-rc.2#[method]gpu-device.create-compute-pipeline`
 * (`(borrow gpu-device, gpu-compute-pipeline-descriptor) -> own gpu-compute-pipeline`;
 * Guest passes shader borrow + layout auto; drops the own; `run` returns 1).
 * L2 still host-fixed stub WGSL + empty layout.
 * via [ExperimentalWebGpuBridge.attachCreateComputePipeline] + [callRunConcurrent].
 * Not full wasi:webgpu compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodCreateComputePipelineInstrumentedTest {
    @Test
    fun guestCreateComputePipelineViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_create_compute_pipeline.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachCreateComputePipeline(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals(
                                    "guest must drop own<gpu-compute-pipeline> and return harness 1",
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
