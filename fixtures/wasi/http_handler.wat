;; WASI 0.3 package smoke: wasi:http incoming-handler (in-process ABI)
;; Official packages: wasi:http/types@0.3.0 + export incoming-handler@0.3.0.
;; Subset: constructors + status-code; handle is async func(own<request>) ->
;; result<own<response>, error-code> (ok path; not outparam / body).
;; Not a listening HTTP server.
;; Guest: construct request → handle ok → status-code 200.
;; Root `run: async func() -> u32` harness returns 200 for callRunConcurrent.
(component
  (import "wasi:http/types@0.3.0" (instance $types
    (export "request" (type $request (sub resource)))
    (export "response" (type $response (sub resource)))
    (export "[constructor]request" (func (result (own $request))))
    (export "[constructor]response" (func (result (own $response))))
    (type $borrow-resp (borrow $response))
    (export "[method]response.status-code"
      (func (param "self" $borrow-resp) (result u16)))
  ))
  (alias export $types "request" (type $request))
  (alias export $types "response" (type $response))
  (alias export $types "[constructor]request" (func $request-ctor))
  (alias export $types "[constructor]response" (func $response-ctor))
  (alias export $types "[method]response.status-code" (func $status-code))

  (type $error-code-def (enum "unknown"))
  (type $handle-result (result (own $response) (error $error-code-def)))

  (core module $libc
    (memory (export "mem") 1)
  )
  (core instance $libc (instantiate $libc))

  (core func $request_ctor_lower (canon lower (func $request-ctor)))
  (core func $response_ctor_lower (canon lower (func $response-ctor)))
  (core func $status_lower (canon lower (func $status-code)))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "request-ctor" (func $request-ctor (result i32)))
    (import "" "response-ctor" (func $response-ctor (result i32)))
    (import "" "status-code" (func $status-code (param i32) (result i32)))

    ;; Own response handle (used by root `run`).
    (func $handle-ok (param $req i32) (result i32)
      (local.get $req)
      drop
      (call $response-ctor)
    )
    ;; Lifted handle returns a pointer to result {u8 disc, i32 handle} at mem[16].
    (func $handle (export "handle") (param $req i32) (result i32)
      (i32.store8 (i32.const 16) (i32.const 0))
      (i32.store (i32.const 20) (call $handle-ok (local.get $req)))
      (i32.const 16)
    )
    (func (export "run") (result i32)
      (call $status-code (call $handle-ok (call $request-ctor)))
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "request-ctor" (func $request_ctor_lower))
      (export "response-ctor" (func $response_ctor_lower))
      (export "status-code" (func $status_lower))
    ))
  ))

  (func $handle-lift async (param "request" (own $request)) (result $handle-result)
    (canon lift (core func $i "handle") (memory $libc "mem")))
  (instance $handler
    (export "error-code" (type $error-code-def))
    (export "handle" (func $handle-lift)))
  (export "wasi:http/incoming-handler@0.3.0" (instance $handler))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
