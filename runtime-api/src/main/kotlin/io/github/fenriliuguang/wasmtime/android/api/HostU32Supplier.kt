package io.github.fenriliuguang.wasmtime.android.api

/** Sync host import callback: `func() -> u32`. */
fun interface HostU32Supplier {
    fun invoke(): Int
}
