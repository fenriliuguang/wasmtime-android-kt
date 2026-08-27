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
 * S6+ `[method]` slice: guest imports `get-device` then
 * `wasi:webgpu/webgpu@0.3.0-rc.2#[method]gpu-device.create-bind-group-layout`
 * (`(borrow gpu-device, gpu-bind-group-layout-descriptor) -> own gpu-bind-group-layout`;
 * Guest passes two buffer entries (binding=0 uniform, binding=1 storage);
 * drops the own; `run` returns 1). L2 described all entries.
 * via [ExperimentalWebGpuBridge.attachCreateBindGroupLayout] + [callRunConcurrent].
 * Not full wasi:webgpu compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodCreateBindGroupLayoutInstrumentedTest {
    @Test
    fun guestCreateBindGroupLayoutViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_create_bind_group_layout.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.createWithFixtureConstructors(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachCreateBindGroupLayout(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals(
                                    "guest must drop own<gpu-bind-group-layout> and return harness 1",
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
