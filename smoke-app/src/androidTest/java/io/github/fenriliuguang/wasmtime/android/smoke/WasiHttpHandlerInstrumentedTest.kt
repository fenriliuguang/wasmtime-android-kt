package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.ByteArrayOutputStream
import java.net.Inet4Address
import java.net.NetworkInterface
import java.net.ServerSocket
import java.util.concurrent.atomic.AtomicReference
import kotlin.concurrent.thread

/**
 * WASI 0.3: `wasi:http` incoming-handler, body `stream<u8>`, and P010 outbound `client.send`.
 * In-process ABI plus wire HTTP/1.1 GET (local non-loopback). Requires `INTERNET`.
 */
@RunWith(AndroidJUnit4::class)
class WasiHttpHandlerInstrumentedTest {
    @Test
    fun incomingHandlerRunReturns200() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/http_handler.wasm")
                .use { it.readBytes() }

        Engine.create().use { engine ->
            Component.compile(engine, bytes).use { component ->
                Linker.create(engine).use { linker ->
                    Store.create(engine).use { store ->
                        linker.instantiate(store, component).use { instance ->
                            assertEquals(200, instance.callRunConcurrent(store))
                        }
                    }
                }
            }
        }
    }

    @Test
    fun bodyStreamEchoReturnsByteCount() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/http_body.wasm")
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
    fun outboundSendHitsBoundPeer() {
        val ip = firstNonLoopbackIpv4()
        assertFalse(ip.isLoopbackAddress)
        val received = AtomicReference<ByteArray>()
        val server = ServerSocket(0, 1, ip).apply { soTimeout = 8_000 }
        val port = server.localPort
        val authority = "${ip.hostAddress}:$port"
        val serverThread =
            thread(name = "wasi-http-out") {
                server.use { listener ->
                    val sock = listener.accept()
                    sock.soTimeout = 8_000
                    sock.use { peer ->
                        val buf = ByteArrayOutputStream()
                        val tmp = ByteArray(512)
                        while (true) {
                            val n = peer.getInputStream().read(tmp)
                            if (n <= 0) break
                            buf.write(tmp, 0, n)
                            val bytes = buf.toByteArray()
                            if (bytes.size >= 4) {
                                val s = bytes.size
                                if (bytes[s - 4] == 0x0d.toByte() &&
                                    bytes[s - 3] == 0x0a.toByte() &&
                                    bytes[s - 2] == 0x0d.toByte() &&
                                    bytes[s - 1] == 0x0a.toByte()
                                ) {
                                    break
                                }
                            }
                        }
                        received.set(buf.toByteArray())
                        peer.getOutputStream().write(
                            "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nHOUT"
                                .toByteArray(),
                        )
                        peer.getOutputStream().flush()
                    }
                }
            }

        val wasm =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("wasi/http_out.wasm")
                .use { it.readBytes() }
        val patched = patchP3Ha(wasm, authority)

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

        val seen = received.get() ?: byteArrayOf()
        assertTrue(
            "host must wire-send GET (peer saw ${seen.decodeToString()})",
            seen.decodeToString().startsWith("GET / HTTP/1.1"),
        )
    }

    companion object {
        private val P3HA = byteArrayOf(0x50, 0x33, 0x48, 0x41)

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

        fun patchP3Ha(wasm: ByteArray, authority: String): ByteArray {
            val bytes = authority.toByteArray(Charsets.US_ASCII)
            require(bytes.size <= 21) { "authority too long" }
            val idx =
                wasm.indices.firstOrNull { i ->
                    i + 4 + 21 < wasm.size &&
                        wasm[i] == P3HA[0] &&
                        wasm[i + 1] == P3HA[1] &&
                        wasm[i + 2] == P3HA[2] &&
                        wasm[i + 3] == P3HA[3]
                } ?: error("P3HA marker missing")
            val out = wasm.copyOf()
            out[idx + 4] = bytes.size.toByte()
            System.arraycopy(bytes, 0, out, idx + 5, bytes.size)
            return out
        }
    }
}
