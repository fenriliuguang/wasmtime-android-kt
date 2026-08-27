;; WASI 0.3: product-path incoming-handler without [constructor]request/response
;; (P010-HCTOR). Host supplies request when calling handle. Guest creates the
;; response via [static]response.new (empty body stream). Root `run` harness
;; returns 200 (status-code) so callRunConcurrent works on Linker.create.
(component
  (import "wasi:http/types@0.3.0" (instance $types
    (export "request" (type $request (sub resource)))
    (export "response" (type $response (sub resource)))
    (type $error-code-def (enum "unknown"))
    (export "error-code" (type $error-code (eq $error-code-def)))
    (type $io-result (result (error $error-code)))
    (type $st (stream u8))
    (type $ft (future $io-result))
    (type $new-ret (tuple (own $response) $ft))
    (type $borrow-resp (borrow $response))
    (export "[static]response.new"
      (func (param "contents" $st) (result $new-ret)))
    (export "[method]response.status-code"
      (func (param "self" $borrow-resp) (result u16)))
  ))
  (alias export $types "request" (type $request))
  (alias export $types "response" (type $response))
  (alias export $types "error-code" (type $error-code))
  (alias export $types "[static]response.new" (func $response-new))
  (alias export $types "[method]response.status-code" (func $status-code))
  (type $io-result (result (error $error-code)))
  (type $st (stream u8))
  (type $ft (future $io-result))
  (type $handle-result (result (own $response) (error $error-code)))

  (core module $libc
    (memory (export "mem") 1)
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.new" (func $stream.new (result i64)))
    (import "" "stream.drop-writable" (func $stream.drop-writable (param i32)))
    (import "" "future.read" (func $future.read (param i32 i32) (result i32)))
    (import "" "future.drop-readable" (func $future.drop-readable (param i32)))
    (import "" "response-new" (func $response-new (param i32 i32)))
    (import "" "status-code" (func $status-code (param i32) (result i32)))

    (func $empty-response (result i32)
      (local $pair i64)
      (local $r i32)
      (local $w i32)
      (local $fut i32)
      (local $status i32)
      (local.set $pair (call $stream.new))
      (local.set $r (i32.wrap_i64 (local.get $pair)))
      (local.set $w (i32.wrap_i64 (i64.shr_u (local.get $pair) (i64.const 32))))
      (call $response-new (local.get $r) (i32.const 80))
      (call $stream.drop-writable (local.get $w))
      (local.set $fut (i32.load (i32.const 84)))
      (local.set $status (call $future.read (local.get $fut) (i32.const 0)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))
      (if (i32.ne (i32.load8_u (i32.const 0)) (i32.const 0))
        (then unreachable))
      (i32.load (i32.const 80))
    )

    (func (export "handle") (param $req i32) (result i32)
      (local.get $req)
      drop
      (i32.store8 (i32.const 16) (i32.const 0))
      (i32.store (i32.const 20) (call $empty-response))
      (i32.const 16)
    )

    (func (export "run") (result i32)
      (call $status-code (call $empty-response))
    )
  )

  (core func $stream.new (canon stream.new $st))
  (core func $stream.drop-writable (canon stream.drop-writable $st))
  (core func $future.read (canon future.read $ft async (memory $libc "mem")))
  (core func $future.drop-readable (canon future.drop-readable $ft))
  (core func $response_new_lower
    (canon lower (func $response-new) (memory $libc "mem")))
  (core func $status_lower (canon lower (func $status-code)))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.new" (func $stream.new))
      (export "stream.drop-writable" (func $stream.drop-writable))
      (export "future.read" (func $future.read))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "response-new" (func $response_new_lower))
      (export "status-code" (func $status_lower))
    ))
  ))

  (func $handle-lift async (param "request" (own $request)) (result $handle-result)
    (canon lift (core func $i "handle") (memory $libc "mem")))
  (instance $handler
    (export "error-code" (type $error-code))
    (export "handle" (func $handle-lift)))
  (export "wasi:http/incoming-handler@0.3.0" (instance $handler))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
