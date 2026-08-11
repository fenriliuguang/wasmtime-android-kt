package io.github.fenriliuguang.wasmtime.android.webgpu

import io.github.fenriliuguang.wasi.webgpu.experimental.abicm.AbiCmHostBindings
import io.github.fenriliuguang.wasi.webgpu.experimental.host.WasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.Store

/**
 * Wire Track A L2 ([WasiWebGpuHost]) into Track B L1 store callbacks for the
 * experimental CM host interface (`AbiCm.IMPORT_INTERFACE`).
 *
 * M3 slice: only [AbiCmHostBindings.requestAdapter].
 */
object ExperimentalWebGpuBridge {
    fun attachRequestAdapter(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setRequestAdapter { bindings.requestAdapter() }
    }
}
