;; L-SOCK-UDP: wasi:sockets/udp@0.3.0 create + bind + send + receive
;; Default sandbox: 127.0.0.1 only. Guest imports sync WIT (no stackful async).
;; send/receive on a helper thread, not ART main.
;; Guest: create ipv4 → bind 127.0.0.1:0 → send "P3UD" to mem[20] port → receive echo.
;; Harness returns 4 on match.
(component
  (import "wasi:sockets/udp@0.3.0" (instance $udp
    (export "udp-socket" (type $udp-socket (sub resource)))
    (type $error-code-def (variant
      (case "access-denied")
      (case "not-supported")
      (case "invalid-argument")
      (case "out-of-memory")
      (case "timeout")
      (case "invalid-state")
      (case "address-not-bindable")
      (case "address-in-use")
      (case "remote-unreachable")
      (case "connection-refused")
      (case "connection-broken")
      (case "connection-reset")
      (case "connection-aborted")
      (case "datagram-too-large")
      (case "other" (option string))
    ))
    (export "error-code" (type $error-code (eq $error-code-def)))
    (type $io-result (result (error $error-code)))
    (type $borrow-sock (borrow $udp-socket))
    (type $ipv4-addr-def (tuple u8 u8 u8 u8))
    (export "ipv4-address" (type $ipv4-addr (eq $ipv4-addr-def)))
    (type $ipv4-sock-def (record (field "port" u16) (field "address" $ipv4-addr)))
    (export "ipv4-socket-address" (type $ipv4-sock (eq $ipv4-sock-def)))
    (type $ip-sock-def (variant (case "ipv4" $ipv4-sock)))
    (export "ip-socket-address" (type $ip-sock (eq $ip-sock-def)))
    (type $opt-ip (option $ip-sock))
    (type $datagram (tuple (list u8) $ip-sock))
    (type $recv-result (result $datagram (error $error-code)))
    (export "[method]udp-socket.bind"
      (func (param "self" $borrow-sock) (param "local-address" $ip-sock) (result $io-result)))
    (export "[method]udp-socket.send"
      (func (param "self" $borrow-sock) (param "data" (list u8)) (param "remote-address" $opt-ip) (result $io-result)))
    (export "[method]udp-socket.receive"
      (func (param "self" $borrow-sock) (result $recv-result)))
  ))
  (alias export $udp "udp-socket" (type $udp-socket))
  (alias export $udp "error-code" (type $error-code))
  (alias export $udp "[method]udp-socket.bind" (func $bind))
  (alias export $udp "[method]udp-socket.send" (func $send))
  (alias export $udp "[method]udp-socket.receive" (func $receive))
  (import "wasi:sockets/udp-create-socket@0.3.0" (instance $create
    (export "udp-socket" (type (eq $udp-socket)))
    (type $error-code-def (variant
      (case "access-denied")
      (case "not-supported")
      (case "invalid-argument")
      (case "out-of-memory")
      (case "timeout")
      (case "invalid-state")
      (case "address-not-bindable")
      (case "address-in-use")
      (case "remote-unreachable")
      (case "connection-refused")
      (case "connection-broken")
      (case "connection-reset")
      (case "connection-aborted")
      (case "datagram-too-large")
      (case "other" (option string))
    ))
    (export "error-code" (type $ec (eq $error-code-def)))
    (type $family-def (enum "ipv4" "ipv6"))
    (export "ip-address-family" (type $family (eq $family-def)))
    (type $create-result (result (own $udp-socket) (error $ec)))
    (export "create-udp-socket"
      (func (param "address-family" $family) (result $create-result)))
  ))
  (alias export $create "create-udp-socket" (func $create-udp-socket))

  (core module $libc
    (memory (export "mem") 1)
    (global $last (mut i32) (i32.const 256))
    (func (export "realloc")
      (param $oldptr i32) (param $oldlen i32) (param $align i32) (param $newlen i32)
      (result i32)
      (local $ret i32)
      (local.set $ret (global.get $last))
      (global.set $last
        (i32.and
          (i32.add (i32.add (local.get $ret) (local.get $newlen)) (i32.const 7))
          (i32.const -8)))
      (local.get $ret)
    )
    (data (i32.const 16) "P3UD\00\00")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "create-udp-socket" (func $create-udp-socket (param i32 i32)))
    (import "" "bind" (func $bind (param i32 i32 i32 i32 i32 i32 i32 i32)))
    (import "" "send" (func $send (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)))
    (import "" "receive" (func $receive (param i32 i32)))

    (func (export "run") (result i32)
      (local $sock i32)
      (local $ptr i32)
      (local $n i32)

      (call $create-udp-socket (i32.const 0) (i32.const 80))
      (if (i32.ne (i32.load8_u (i32.const 80)) (i32.const 0))
        (then (return (i32.const 0))))
      (local.set $sock (i32.load (i32.const 84)))

      ;; bind ipv4 127.0.0.1:0
      (call $bind
        (local.get $sock)
        (i32.const 0)
        (i32.const 0)
        (i32.const 127)
        (i32.const 0)
        (i32.const 0)
        (i32.const 1)
        (i32.const 192))
      (if (i32.ne (i32.load8_u (i32.const 192)) (i32.const 0))
        (then (return (i32.const 0))))

      ;; send "P3UD" to 127.0.0.1:port (mem[20])
      (call $send
        (local.get $sock)
        (i32.const 16)
        (i32.const 4)
        (i32.const 1)
        (i32.const 0)
        (i32.load16_u (i32.const 20))
        (i32.const 127)
        (i32.const 0)
        (i32.const 0)
        (i32.const 1)
        (i32.const 192))
      (if (i32.ne (i32.load8_u (i32.const 192)) (i32.const 0))
        (then (return (i32.const 0))))

      (call $receive (local.get $sock) (i32.const 192))
      (if (i32.ne (i32.load8_u (i32.const 192)) (i32.const 0))
        (then (return (i32.const 0))))
      (local.set $ptr (i32.load (i32.const 196)))
      (local.set $n (i32.load (i32.const 200)))
      (if (i32.ne (local.get $n) (i32.const 4))
        (then (return (i32.const 0))))
      (if (i32.ne (i32.load (local.get $ptr)) (i32.load (i32.const 16)))
        (then (return (i32.const 0))))

      (i32.const 4)
    )
  )

  (core func $create_lower
    (canon lower (func $create-udp-socket)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $bind_lower
    (canon lower (func $bind)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $send_lower
    (canon lower (func $send)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $receive_lower
    (canon lower (func $receive)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "create-udp-socket" (func $create_lower))
      (export "bind" (func $bind_lower))
      (export "send" (func $send_lower))
      (export "receive" (func $receive_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
