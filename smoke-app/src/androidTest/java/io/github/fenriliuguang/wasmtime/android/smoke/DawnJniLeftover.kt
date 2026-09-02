package io.github.fenriliuguang.wasmtime.android.smoke

import org.junit.Assume

/**
 * Leftover androidx `dawn-jni` path. Product packaging excludes
 * `libwebgpu_c_bundled.so`; skip those instruments when the .so is absent.
 */
internal object DawnJniLeftover {
    fun assumeBundledLibrary() {
        Assume.assumeTrue(
            "dawn-jni leftover needs libwebgpu_c_bundled.so (excluded from the product APK)",
            isBundledLibraryPresent(),
        )
    }

    private fun isBundledLibraryPresent(): Boolean =
        try {
            System.loadLibrary("webgpu_c_bundled")
            true
        } catch (_: UnsatisfiedLinkError) {
            false
        }
}
