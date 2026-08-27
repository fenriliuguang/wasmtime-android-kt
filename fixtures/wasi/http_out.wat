;; WASI 0.3 package smoke: wasi:http client.send outbound HTTP/1.1 (P010-HOUT)
;; Official: wasi:http/client@0.3.0#send (0.3 equivalent of outgoing-handler).
;; Guest: ctor request → set-authority(P3HA host:port) → send → status 200
;; → consume-body "HOUT" → nbytes 4. Tests patch P3HA before instantiate.
;; Wire GET (not in-process 200). No TLS / headers / trailers.
(component
  (import "wasi:http/types@0.3.0" (instance $types
    (export "request" (type $request (sub resource)))
    (export "response" (type $response (sub resource)))
    (type $error-code-def (enum "unknown"))
    (export "error-code" (type $error-code (eq $error-code-def)))
    (type $io-result (result (error $error-code)))
    (type $st (stream u8))
    (type $ft (future $io-result))
    (type $read-ret (tuple $st $ft))
    (type $borrow-req (borrow $request))
    (type $borrow-resp (borrow $response))
    (export "[constructor]request" (func (result (own $request))))
    (export "[method]request.set-authority"
      (func (param "self" $borrow-req) (param "authority" string) (result $io-result)))
    (export "[method]response.status-code"
      (func (param "self" $borrow-resp) (result u16)))
    (export "[static]response.consume-body"
      (func (param "this" (own $response)) (result $read-ret)))
  ))
  (alias export $types "request" (type $request))
  (alias export $types "response" (type $response))
  (alias export $types "error-code" (type $error-code))
  (alias export $types "[constructor]request" (func $request-ctor))
  (alias export $types "[method]request.set-authority" (func $set-authority))
  (alias export $types "[method]response.status-code" (func $status-code))
  (alias export $types "[static]response.consume-body" (func $resp-consume))
  (type $io-result (result (error $error-code)))
  (type $st (stream u8))
  (type $ft (future $io-result))
  (type $send-result (result (own $response) (error $error-code)))

  (import "wasi:http/client@0.3.0" (instance $client
    (export "request" (type (eq $request)))
    (export "response" (type (eq $response)))
    (export "error-code" (type (eq $error-code)))
    (export "send"
      (func async (param "request" (own $request)) (result $send-result)))
  ))
  (alias export $client "send" (func $send))

  (core module $libc
    (memory (export "mem") 1)
    (data (i32.const 16) "HOUT")
    ;; P3HA + len + 21-byte authority pad (one segment so tests can patch the wasm)
    (data (i32.const 160) "P3HA\00.....................")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.read" (func $stream.read (param i32 i32 i32) (result i32)))
    (import "" "future.read" (func $future.read (param i32 i32) (result i32)))
    (import "" "future.drop-readable" (func $future.drop-readable (param i32)))
    (import "" "request-ctor" (func $request-ctor (result i32)))
    (import "" "set-authority" (func $set-authority (param i32 i32 i32 i32)))
    (import "" "status-code" (func $status-code (param i32) (result i32)))
    (import "" "send" (func $send (param i32 i32)))
    (import "" "resp-consume" (func $resp-consume (param i32 i32)))

    (func (export "run") (result i32)
      (local $req i32)
      (local $resp i32)
      (local $len i32)
      (local $s i32)
      (local $fut i32)
      (local $status i32)
      (local $n i32)

      (local.set $req (call $request-ctor))
      (local.set $len (i32.load8_u (i32.const 164)))
      (call $set-authority
        (local.get $req)
        (i32.const 165)
        (local.get $len)
        (i32.const 96))
      (if (i32.ne (i32.load8_u (i32.const 96)) (i32.const 0))
        (then unreachable))

      (call $send (local.get $req) (i32.const 32))
      (if (i32.ne (i32.load8_u (i32.const 32)) (i32.const 0))
        (then unreachable))
      (local.set $resp (i32.load (i32.const 36)))
      (if (i32.ne (call $status-code (local.get $resp)) (i32.const 200))
        (then unreachable))

      (call $resp-consume (local.get $resp) (i32.const 48))
      (local.set $s (i32.load (i32.const 48)))
      (local.set $fut (i32.load (i32.const 52)))

      (local.set $status (call $stream.read (local.get $s) (i32.const 64) (i32.const 16)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (local.set $n (i32.shr_u (local.get $status) (i32.const 4)))
      (if (i32.ne (i32.load (i32.const 64)) (i32.load (i32.const 16)))
        (then unreachable))

      (local.set $status (call $future.read (local.get $fut) (i32.const 80)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))
      (if (i32.ne (i32.load8_u (i32.const 80)) (i32.const 0))
        (then unreachable))

      (local.get $n)
    )
  )

  (core func $stream.read (canon stream.read $st async (memory $libc "mem")))
  (core func $future.read (canon future.read $ft async (memory $libc "mem")))
  (core func $future.drop-readable (canon future.drop-readable $ft))
  (core func $request_ctor_lower (canon lower (func $request-ctor)))
  (core func $set_authority_lower
    (canon lower (func $set-authority) (memory $libc "mem")))
  (core func $status_lower (canon lower (func $status-code)))
  (core func $send_lower (canon lower (func $send) (memory $libc "mem")))
  (core func $resp_consume_lower
    (canon lower (func $resp-consume) (memory $libc "mem")))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.read" (func $stream.read))
      (export "future.read" (func $future.read))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "request-ctor" (func $request_ctor_lower))
      (export "set-authority" (func $set_authority_lower))
      (export "status-code" (func $status_lower))
      (export "send" (func $send_lower))
      (export "resp-consume" (func $resp_consume_lower))
    ))
  ))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
