package io.github.fenriliuguang.wasmtime.android.smoke

import android.system.Os
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

/**
 * WASI 0.3: `wasi:filesystem` preopen + read/write via guest `run`.
 * `get-directories` is official `list<tuple<descriptor, string>>` (index 0 = sandbox dir).
 * Guest `open-at("p3fs.txt")` then r/w-via-stream at `offset` 0.
 * Sandbox is app-private cache (`TMPDIR`), not shared storage.
 * Guest writes `P3FS` then reads it back (4 bytes).
 */
@RunWith(AndroidJUnit4::class)
class WasiFilesystemPreopenInstrumentedTest {
    @Test
    fun preopenWriteThenReadReturnsByteCount() {
        val cacheDir =
            InstrumentationRegistry.getInstrumentation().targetContext.cacheDir
        Os.setenv("TMPDIR", cacheDir.absolutePath, true)

        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/filesystem_preopen.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(4, instance.callStreamWrite(store))
                        }
                    }
                }
            }
        }
    }
}
