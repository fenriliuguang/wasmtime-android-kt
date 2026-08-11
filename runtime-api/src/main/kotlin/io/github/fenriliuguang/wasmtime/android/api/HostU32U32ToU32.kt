package io.github.fenriliuguang.wasmtime.android.api

/** Sync host import callback: `(u32, u32) -> u32`. */
fun interface HostU32U32ToU32 {
    fun invoke(a: Int, b: Int): Int
}
