;; WASI 0.3 package smoke: wasi:http incoming-handler (in-process ABI)
;; Official packages: wasi:http/types@0.3.0 + export incoming-handler@0.3.0.
;; Subset: constructors + status-code; handle is async func(own<request>) ->
;; own<response> (not result / outparam / body). Not a listening HTTP server.
;; Guest: construct request → handle → status-code 200.
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

  (core func $request_ctor_lower (canon lower (func $request-ctor)))
  (core func $response_ctor_lower (canon lower (func $response-ctor)))
  (core func $status_lower (canon lower (func $status-code)))

  (core module $m
    (import "" "request-ctor" (func $request-ctor (result i32)))
    (import "" "response-ctor" (func $response-ctor (result i32)))
    (import "" "status-code" (func $status-code (param i32) (result i32)))

    (func $handle (export "handle") (param $req i32) (result i32)
      (local.get $req)
      drop
      (call $response-ctor)
    )
    (func (export "run") (result i32)
      (call $status-code (call $handle (call $request-ctor)))
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "request-ctor" (func $request_ctor_lower))
      (export "response-ctor" (func $response_ctor_lower))
      (export "status-code" (func $status_lower))
    ))
  ))

  (func $handle-lift async (param "request" (own $request)) (result (own $response))
    (canon lift (core func $i "handle")))
  (instance $handler
    (export "handle" (func $handle-lift)))
  (export "wasi:http/incoming-handler@0.3.0" (instance $handler))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
