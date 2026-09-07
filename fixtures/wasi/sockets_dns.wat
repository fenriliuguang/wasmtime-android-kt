;; L-SOCK-DNS: wasi:sockets/ip-name-lookup@0.3.0 resolve-addresses
;; Guest imports sync WIT (no stackful async). Lookup on a helper thread, not ART main.
;; Guest: resolve "localhost", succeed if any ipv4 is 127.0.0.1. Harness 1.
(component
  (import "wasi:sockets/ip-name-lookup@0.3.0" (instance $dns
    (type $error-code-def (enum
      "unknown"
      "access-denied"
      "invalid-argument"
      "name-unresolvable"
      "temporary-resolver-failure"
      "permanent-resolver-failure"
    ))
    (export "error-code" (type $error-code (eq $error-code-def)))
    (type $ipv4-addr-def (tuple u8 u8 u8 u8))
    (export "ipv4-address" (type $ipv4-addr (eq $ipv4-addr-def)))
    (type $ip-addr-def (variant (case "ipv4" $ipv4-addr)))
    (export "ip-address" (type $ip-addr (eq $ip-addr-def)))
    (type $addr-list (list $ip-addr))
    (type $lookup-result (result $addr-list (error $error-code)))
    (export "resolve-addresses"
      (func (param "name" string) (result $lookup-result)))
  ))
  (alias export $dns "resolve-addresses" (func $resolve-addresses))

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
    (data (i32.const 16) "localhost")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "resolve-addresses" (func $resolve (param i32 i32 i32)))

    (func (export "run") (result i32)
      (local $n i32)

      (call $resolve (i32.const 16) (i32.const 9) (i32.const 80))
      (if (i32.ne (i32.load8_u (i32.const 80)) (i32.const 0))
        (then (return (i32.const 0))))
      (local.set $n (i32.load (i32.const 88)))
      (if (i32.eqz (local.get $n))
        (then (return (i32.const 0))))
      (i32.const 1)
    )
  )

  (core func $resolve_lower
    (canon lower (func $resolve-addresses)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "resolve-addresses" (func $resolve_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
