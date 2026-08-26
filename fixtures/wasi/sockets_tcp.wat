;; WASI 0.3 package smoke: wasi:sockets TCP loopback echo
;; Official packages: wasi:sockets/tcp@0.3.0 + tcp-create-socket@0.3.0.
;; create-tcp-socket(ip-address-family) -> result<tcp-socket, error-code> (ipv4 ok).
;; connect: async func(ip-socket-address) -> result (guest loopback; host may ignore port).
;; write/read via streams. No UDP / listen / ip-name-lookup.
;; Guest: create ipv4 → connect 127.0.0.1 → write "P3SK" → read echo → nbytes 4.
(component
  (import "wasi:sockets/tcp@0.3.0" (instance $tcp
    (export "tcp-socket" (type $tcp-socket (sub resource)))
    (type $error-code-def (enum "unknown"))
    (export "error-code" (type $error-code (eq $error-code-def)))
    (type $io-result (result (error $error-code)))
    (type $st (stream u8))
    (type $ft (future $io-result))
    (type $read-ret (tuple $st $ft))
    (type $borrow-sock (borrow $tcp-socket))
    (type $ipv4-addr-def (tuple u8 u8 u8 u8))
    (export "ipv4-address" (type $ipv4-addr (eq $ipv4-addr-def)))
    (type $ipv4-sock-def (record (field "port" u16) (field "address" $ipv4-addr)))
    (export "ipv4-socket-address" (type $ipv4-sock (eq $ipv4-sock-def)))
    (type $ip-sock-def (variant (case "ipv4" $ipv4-sock)))
    (export "ip-socket-address" (type $ip-sock (eq $ip-sock-def)))
    (export "[method]tcp-socket.connect"
      (func async (param "self" $borrow-sock) (param "remote-address" $ip-sock) (result $io-result)))
    (export "[method]tcp-socket.write-via-stream"
      (func (param "self" $borrow-sock) (param "data" $st) (result $ft)))
    (export "[method]tcp-socket.read-via-stream"
      (func (param "self" $borrow-sock) (result $read-ret)))
  ))
  (alias export $tcp "tcp-socket" (type $tcp-socket))
  (alias export $tcp "error-code" (type $error-code))
  (alias export $tcp "[method]tcp-socket.connect" (func $connect))
  (alias export $tcp "[method]tcp-socket.write-via-stream" (func $write-via-stream))
  (alias export $tcp "[method]tcp-socket.read-via-stream" (func $read-via-stream))
  (import "wasi:sockets/tcp-create-socket@0.3.0" (instance $create
    (export "tcp-socket" (type (eq $tcp-socket)))
    (type $error-code-def (enum "unknown"))
    (export "error-code" (type $ec (eq $error-code-def)))
    (type $family-def (enum "ipv4" "ipv6"))
    (export "ip-address-family" (type $family (eq $family-def)))
    (type $create-result (result (own $tcp-socket) (error $ec)))
    (export "create-tcp-socket"
      (func (param "address-family" $family) (result $create-result)))
  ))
  (alias export $create "create-tcp-socket" (func $create-tcp-socket))
  (type $io-result (result (error $error-code)))
  (type $st (stream u8))
  (type $ft (future $io-result))

  (core module $libc
    (memory (export "mem") 1)
    (data (i32.const 16) "P3SK")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.new" (func $stream.new (result i64)))
    (import "" "stream.write" (func $stream.write (param i32 i32 i32) (result i32)))
    (import "" "stream.read" (func $stream.read (param i32 i32 i32) (result i32)))
    (import "" "stream.drop-writable" (func $stream.drop-writable (param i32)))
    (import "" "future.read" (func $future.read (param i32 i32) (result i32)))
    (import "" "future.drop-readable" (func $future.drop-readable (param i32)))
    (import "" "create-tcp-socket" (func $create-tcp-socket (param i32 i32)))
    (import "" "connect" (func $connect (param i32 i32 i32 i32 i32 i32 i32 i32)))
    (import "" "write-via-stream" (func $write-via-stream (param i32 i32) (result i32)))
    (import "" "read-via-stream" (func $read-via-stream (param i32 i32)))

    (func (export "run") (result i32)
      (local $sock i32)
      (local $pair i64)
      (local $r i32)
      (local $w i32)
      (local $fut i32)
      (local $s i32)
      (local $status i32)
      (local $n i32)

      ;; result at mem[80]: u8 disc (0=ok), handle at 84. family ipv4 = 0.
      (call $create-tcp-socket (i32.const 0) (i32.const 80))
      (if (i32.ne (i32.load8_u (i32.const 80)) (i32.const 0))
        (then unreachable))
      (local.set $sock (i32.load (i32.const 84)))
      ;; connect(self, ipv4 {port=0, 127.0.0.1}, retptr=144)
      (call $connect
        (local.get $sock)
        (i32.const 0)
        (i32.const 0)
        (i32.const 127)
        (i32.const 0)
        (i32.const 0)
        (i32.const 1)
        (i32.const 144))
      (if (i32.ne (i32.load8_u (i32.const 144)) (i32.const 0))
        (then unreachable))

      (local.set $pair (call $stream.new))
      (local.set $r (i32.wrap_i64 (local.get $pair)))
      (local.set $w (i32.wrap_i64 (i64.shr_u (local.get $pair) (i64.const 32))))
      (local.set $fut (call $write-via-stream (local.get $sock) (local.get $r)))

      (local.set $status (call $stream.write (local.get $w) (i32.const 16) (i32.const 4)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $stream.drop-writable (local.get $w))

      (local.set $status (call $future.read (local.get $fut) (i32.const 0)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))
      (if (i32.ne (i32.load8_u (i32.const 0)) (i32.const 0))
        (then unreachable))

      ;; tuple at mem[32]: stream handle, future handle
      (call $read-via-stream (local.get $sock) (i32.const 32))
      (local.set $s (i32.load (i32.const 32)))
      (local.set $fut (i32.load (i32.const 36)))

      (local.set $status (call $stream.read (local.get $s) (i32.const 48) (i32.const 16)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (local.set $n (i32.shr_u (local.get $status) (i32.const 4)))
      (if (i32.ne (i32.load (i32.const 48)) (i32.load (i32.const 16)))
        (then unreachable))

      (local.set $status (call $future.read (local.get $fut) (i32.const 64)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))
      (if (i32.ne (i32.load8_u (i32.const 64)) (i32.const 0))
        (then unreachable))

      (local.get $n)
    )
  )

  (core func $stream.new (canon stream.new $st))
  (core func $stream.write (canon stream.write $st async (memory $libc "mem")))
  (core func $stream.read (canon stream.read $st async (memory $libc "mem")))
  (core func $stream.drop-writable (canon stream.drop-writable $st))
  (core func $future.read (canon future.read $ft async (memory $libc "mem")))
  (core func $future.drop-readable (canon future.drop-readable $ft))
  (core func $create_lower
    (canon lower (func $create-tcp-socket) (memory $libc "mem")))
  (core func $connect_lower
    (canon lower (func $connect) (memory $libc "mem")))
  (core func $write_lower (canon lower (func $write-via-stream) (memory $libc "mem")))
  (core func $read_lower (canon lower (func $read-via-stream) (memory $libc "mem")))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.new" (func $stream.new))
      (export "stream.write" (func $stream.write))
      (export "stream.read" (func $stream.read))
      (export "stream.drop-writable" (func $stream.drop-writable))
      (export "future.read" (func $future.read))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "create-tcp-socket" (func $create_lower))
      (export "connect" (func $connect_lower))
      (export "write-via-stream" (func $write_lower))
      (export "read-via-stream" (func $read_lower))
    ))
  ))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
