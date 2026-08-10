package io.github.fenriliuguang.wasmtime.android.smoke

import android.app.Activity
import android.os.Bundle
import android.widget.TextView
import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge

/** Minimal shell: load native + show Wasmtime version (M0). */
class MainActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val text = TextView(this)
        text.text =
            try {
                val id = NativeBridge.nativeRuntimeId()
                val ver = NativeBridge.nativeWasmtimeVersion()
                "ok\n$id\nwasmtime $ver"
            } catch (t: Throwable) {
                "load failed: ${t.message}"
            }
        setContentView(text)
    }
}
