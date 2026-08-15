package io.github.fenriliuguang.wasmtime.android.smoke

import android.app.Activity
import android.app.KeyguardManager
import android.os.Build
import android.os.Bundle
import android.view.WindowManager
import android.widget.TextView
import io.github.fenriliuguang.wasmtime.android.jni.NativeBridge

/**
 * Smoke shell: SurfaceView for M4 instruments + native version probe.
 *
 * Pass [EXTRA_SKIP_DEMO_AUTORUN] to skip native load (instrumented Surface path).
 */
class MainActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableShowWhenLockedAndTurnScreenOn()
        setContentView(R.layout.activity_main)

        val status = findViewById<TextView>(R.id.status)
        val skipDemo = intent.getBooleanExtra(EXTRA_SKIP_DEMO_AUTORUN, false)
        status.text =
            if (skipDemo) {
                "Surface ready (autorun skipped)"
            } else {
                try {
                    val id = NativeBridge.nativeRuntimeId()
                    val ver = NativeBridge.nativeWasmtimeVersion()
                    "ok\n$id\nwasmtime $ver"
                } catch (t: Throwable) {
                    "load failed: ${t.message}"
                }
            }
    }

    private fun enableShowWhenLockedAndTurnScreenOn() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O_MR1) {
            setShowWhenLocked(true)
            setTurnScreenOn(true)
        } else {
            @Suppress("DEPRECATION")
            window.addFlags(
                WindowManager.LayoutParams.FLAG_SHOW_WHEN_LOCKED or
                    WindowManager.LayoutParams.FLAG_TURN_SCREEN_ON,
            )
        }
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            getSystemService(KeyguardManager::class.java)?.requestDismissKeyguard(this, null)
        }
    }

    companion object {
        const val EXTRA_SKIP_DEMO_AUTORUN: String =
            "io.github.fenriliuguang.wasmtime.android.smoke.SKIP_DEMO_AUTORUN"
    }
}
