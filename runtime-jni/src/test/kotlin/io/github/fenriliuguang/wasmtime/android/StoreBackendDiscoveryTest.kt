package io.github.fenriliuguang.wasmtime.android

import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackend
import io.github.fenriliuguang.wasmtime.android.api.WebGpuBackendFactory
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** P010-DISC: ServiceLoader pick (no JNI). Empty → none; several prefer dawn. */
class StoreBackendDiscoveryTest {
    @Test
    fun emptyFactoriesYieldNull() {
        assertNull(Store.pickDiscoveredBackend(emptyList()))
    }

    @Test
    fun singleFactoryIsUsed() {
        val picked = Store.pickDiscoveredBackend(listOf(FakeFactory("cpu")))
        assertEquals("cpu", picked!!.id)
    }

    @Test
    fun severalFactoriesPreferDawn() {
        val cpu = FakeFactory("cpu")
        val dawn = FakeFactory("dawn")
        val picked = Store.pickDiscoveredBackend(listOf(cpu, dawn))
        assertEquals("dawn", picked!!.id)
        assertTrue(cpu.last!!.closed)
        assertTrue(!dawn.last!!.closed)
    }

    @Test
    fun severalWithoutDawnUseFirstAndCloseRest() {
        val first = FakeFactory("cpu")
        val second = FakeFactory("other")
        val picked = Store.pickDiscoveredBackend(listOf(first, second))
        assertEquals("cpu", picked!!.id)
        assertTrue(!first.last!!.closed)
        assertTrue(second.last!!.closed)
    }

    private class FakeBackend(override val id: String) : WebGpuBackend {
        var closed: Boolean = false

        override fun close() {
            closed = true
        }
    }

    private class FakeFactory(private val id: String) : WebGpuBackendFactory {
        var last: FakeBackend? = null

        override fun create(): WebGpuBackend {
            val backend = FakeBackend(id)
            last = backend
            return backend
        }
    }
}
