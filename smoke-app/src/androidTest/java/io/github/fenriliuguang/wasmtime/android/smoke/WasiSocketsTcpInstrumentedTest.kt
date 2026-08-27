package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test
import org.junit.runner.RunWith
import java.net.Inet4Address
import java.net.NetworkInterface
import java.net.ServerSocket
import java.util.concurrent.atomic.AtomicReference
import kotlin.concurrent.thread

/**
 * WASI 0.3: `wasi:sockets` TCP via guest async `run`.
 * Requires `INTERNET`. Host IO is on a helper thread.
 * Loopback: W7/SK2 echo pair. Outbound: host dials guest non-loopback IPv4:port.
 */
@RunWith(AndroidJUnit4::class)
class WasiSocketsTcpInstrumentedTest {
    @Test
    fun loopbackEchoReturnsByteCount() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/sockets_tcp.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(4, instance.callRunConcurrent(store))
                        }
                    }
                }
            }
        }
    }

    @Test
    fun outboundNonLoopbackEchoHitsBoundPeer() {
        val ip = firstNonLoopbackIpv4()
        assertFalse(ip.isLoopbackAddress)
        val received = AtomicReference<ByteArray>()
        val server =
            ServerSocket(0, 1, ip).apply { soTimeout = 8_000 }
        val port = server.localPort
        val serverThread =
            thread(name = "wasi-tcp-echo") {
                server.use { listener ->
                    val sock = listener.accept()
                    sock.soTimeout = 8_000
                    sock.use { peer ->
                        val buf = peer.getInputStream().readBytes()
                        received.set(buf)
                        peer.getOutputStream().write(buf)
                        peer.getOutputStream().flush()
                    }
                }
            }

        val wasm =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/sockets_tcp_out.wasm")
                .use { it.readBytes() }
        val patched = patchP3Ip(wasm, ip.address, port)

        try {
            Engine.create().use { engine ->
                Component.compile(engine, patched).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            linker.instantiate(store, component).use { instance ->
                                assertEquals(4, instance.callRunConcurrent(store))
                            }
                        }
                    }
                }
            }
        } finally {
            server.close()
            serverThread.join(8_000)
        }

        assertArrayEquals(
            "host must dial guest address (echo server saw no payload if ignore-port pair)",
            byteArrayOf(0x50, 0x33, 0x53, 0x4B), // P3SK
            received.get(),
        )
    }

    companion object {
        private val P3IP = byteArrayOf(0x50, 0x33, 0x49, 0x50)

        fun firstNonLoopbackIpv4(): Inet4Address {
            val ifaces = NetworkInterface.getNetworkInterfaces()
                ?: error("need non-loopback IPv4")
            for (nif in ifaces) {
                if (!nif.isUp || nif.isLoopback) continue
                for (addr in nif.inetAddresses) {
                    if (addr is Inet4Address &&
                        !addr.isLoopbackAddress &&
                        !addr.isAnyLocalAddress
                    ) {
                        return addr
                    }
                }
            }
            error("need non-loopback IPv4")
        }

        fun patchP3Ip(wasm: ByteArray, ipv4: ByteArray, port: Int): ByteArray {
            require(ipv4.size == 4)
            val idx =
                wasm.indices.firstOrNull { i ->
                    i + 9 < wasm.size &&
                        wasm[i] == P3IP[0] &&
                        wasm[i + 1] == P3IP[1] &&
                        wasm[i + 2] == P3IP[2] &&
                        wasm[i + 3] == P3IP[3]
                } ?: error("P3IP marker missing")
            val out = wasm.copyOf()
            out[idx + 4] = (port and 0xff).toByte()
            out[idx + 5] = ((port shr 8) and 0xff).toByte()
            out[idx + 6] = ipv4[0]
            out[idx + 7] = ipv4[1]
            out[idx + 8] = ipv4[2]
            out[idx + 9] = ipv4[3]
            return out
        }
    }
}
