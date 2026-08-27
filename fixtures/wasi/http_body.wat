;; WASI 0.3 package smoke: wasi:http body stream<u8> (P010-HBODY)
;; Official: request/response consume-body → tuple<stream<u8>, future<…>>;
;; response.new(contents: stream<u8>). Subset: no headers / trailers / res-future
;; param. In-process (not a listening HTTP server).
;; Guest: ctor request (host body HBOD) → consume-body read → response.new write
;; → consume-body read echo → nbytes 4.
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
    (type $new-ret (tuple (own $response) $ft))
    (export "[constructor]request" (func (result (own $request))))
    (export "[static]request.consume-body"
      (func (param "this" (own $request)) (result $read-ret)))
    (export "[static]response.new"
      (func (param "contents" $st) (result $new-ret)))
    (export "[static]response.consume-body"
      (func (param "this" (own $response)) (result $read-ret)))
  ))
  (alias export $types "request" (type $request))
  (alias export $types "response" (type $response))
  (alias export $types "error-code" (type $error-code))
  (alias export $types "[constructor]request" (func $request-ctor))
  (alias export $types "[static]request.consume-body" (func $req-consume))
  (alias export $types "[static]response.new" (func $response-new))
  (alias export $types "[static]response.consume-body" (func $resp-consume))
  (type $io-result (result (error $error-code)))
  (type $st (stream u8))
  (type $ft (future $io-result))

  (core module $libc
    (memory (export "mem") 1)
    (data (i32.const 16) "HBOD")
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
    (import "" "request-ctor" (func $request-ctor (result i32)))
    (import "" "req-consume" (func $req-consume (param i32 i32)))
    (import "" "response-new" (func $response-new (param i32 i32)))
    (import "" "resp-consume" (func $resp-consume (param i32 i32)))

    (func (export "run") (result i32)
      (local $req i32)
      (local $resp i32)
      (local $pair i64)
      (local $r i32)
      (local $w i32)
      (local $s i32)
      (local $fut i32)
      (local $status i32)
      (local $n i32)

      (local.set $req (call $request-ctor))
      ;; tuple at mem[32]: stream, future
      (call $req-consume (local.get $req) (i32.const 32))
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

      (local.set $pair (call $stream.new))
      (local.set $r (i32.wrap_i64 (local.get $pair)))
      (local.set $w (i32.wrap_i64 (i64.shr_u (local.get $pair) (i64.const 32))))
      ;; tuple at mem[80]: response, future
      (call $response-new (local.get $r) (i32.const 80))
      (local.set $resp (i32.load (i32.const 80)))
      (local.set $fut (i32.load (i32.const 84)))

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

      (call $resp-consume (local.get $resp) (i32.const 96))
      (local.set $s (i32.load (i32.const 96)))
      (local.set $fut (i32.load (i32.const 100)))

      (local.set $status (call $stream.read (local.get $s) (i32.const 112) (i32.const 16)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (if (i32.ne (i32.load (i32.const 112)) (i32.load (i32.const 16)))
        (then unreachable))

      (local.set $status (call $future.read (local.get $fut) (i32.const 128)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))
      (if (i32.ne (i32.load8_u (i32.const 128)) (i32.const 0))
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
  (core func $request_ctor_lower (canon lower (func $request-ctor)))
  (core func $req_consume_lower
    (canon lower (func $req-consume) (memory $libc "mem")))
  (core func $response_new_lower
    (canon lower (func $response-new) (memory $libc "mem")))
  (core func $resp_consume_lower
    (canon lower (func $resp-consume) (memory $libc "mem")))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.new" (func $stream.new))
      (export "stream.write" (func $stream.write))
      (export "stream.read" (func $stream.read))
      (export "stream.drop-writable" (func $stream.drop-writable))
      (export "future.read" (func $future.read))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "request-ctor" (func $request_ctor_lower))
      (export "req-consume" (func $req_consume_lower))
      (export "response-new" (func $response_new_lower))
      (export "resp-consume" (func $resp_consume_lower))
    ))
  ))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
