;; L-SOCK-LISTEN: wasi:sockets/tcp@0.3.0 bind / listen / accept
;; Default sandbox: 127.0.0.1 only. Guest imports sync WIT (no stackful async).
;; bind/listen/accept on a helper thread, not ART main.
;; Guest: create ipv4 → bind 127.0.0.1:port (mem[16] u16 le) → listen → accept. Harness 1.
(component
  (import "wasi:sockets/tcp@0.3.0" (instance $tcp
    (export "tcp-socket" (type $tcp-socket (sub resource)))
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
    (type $borrow-sock (borrow $tcp-socket))
    (type $ipv4-addr-def (tuple u8 u8 u8 u8))
    (export "ipv4-address" (type $ipv4-addr (eq $ipv4-addr-def)))
    (type $ipv4-sock-def (record (field "port" u16) (field "address" $ipv4-addr)))
    (export "ipv4-socket-address" (type $ipv4-sock (eq $ipv4-sock-def)))
    (type $ip-sock-def (variant (case "ipv4" $ipv4-sock)))
    (export "ip-socket-address" (type $ip-sock (eq $ip-sock-def)))
    (type $accept-ok (tuple (own $tcp-socket) $ip-sock))
    (type $accept-result (result $accept-ok (error $error-code)))
    (export "[method]tcp-socket.bind"
      (func (param "self" $borrow-sock) (param "local-address" $ip-sock) (result $io-result)))
    (export "[method]tcp-socket.listen"
      (func (param "self" $borrow-sock) (result $io-result)))
    (export "[method]tcp-socket.accept"
      (func (param "self" $borrow-sock) (result $accept-result)))
  ))
  (alias export $tcp "tcp-socket" (type $tcp-socket))
  (alias export $tcp "error-code" (type $error-code))
  (alias export $tcp "[method]tcp-socket.bind" (func $bind))
  (alias export $tcp "[method]tcp-socket.listen" (func $listen))
  (alias export $tcp "[method]tcp-socket.accept" (func $accept))
  (import "wasi:sockets/tcp-create-socket@0.3.0" (instance $create
    (export "tcp-socket" (type (eq $tcp-socket)))
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
    (type $create-result (result (own $tcp-socket) (error $ec)))
    (export "create-tcp-socket"
      (func (param "address-family" $family) (result $create-result)))
  ))
  (alias export $create "create-tcp-socket" (func $create-tcp-socket))

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
    (data (i32.const 16) "P3BN\35\49")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "create-tcp-socket" (func $create-tcp-socket (param i32 i32)))
    (import "" "bind" (func $bind (param i32 i32 i32 i32 i32 i32 i32 i32)))
    (import "" "listen" (func $listen (param i32 i32)))
    (import "" "accept" (func $accept (param i32 i32)))

    (func (export "run") (result i32)
      (local $sock i32)

      (call $create-tcp-socket (i32.const 0) (i32.const 80))
      (if (i32.ne (i32.load8_u (i32.const 80)) (i32.const 0))
        (then (return (i32.const 0))))
      (local.set $sock (i32.load (i32.const 84)))

      ;; bind ipv4 127.0.0.1:port
      (call $bind
        (local.get $sock)
        (i32.const 0)
        (i32.load16_u (i32.const 20))
        (i32.const 127)
        (i32.const 0)
        (i32.const 0)
        (i32.const 1)
        (i32.const 192))
      (if (i32.ne (i32.load8_u (i32.const 192)) (i32.const 0))
        (then (return (i32.const 0))))

      (call $listen (local.get $sock) (i32.const 192))
      (if (i32.ne (i32.load8_u (i32.const 192)) (i32.const 0))
        (then (return (i32.const 0))))

      (call $accept (local.get $sock) (i32.const 192))
      (if (i32.ne (i32.load8_u (i32.const 192)) (i32.const 0))
        (then (return (i32.const 0))))

      (i32.const 1)
    )
  )

  (core func $create_lower
    (canon lower (func $create-tcp-socket)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $bind_lower
    (canon lower (func $bind)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $listen_lower
    (canon lower (func $listen)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $accept_lower
    (canon lower (func $accept)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "create-tcp-socket" (func $create_lower))
      (export "bind" (func $bind_lower))
      (export "listen" (func $listen_lower))
      (export "accept" (func $accept_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
