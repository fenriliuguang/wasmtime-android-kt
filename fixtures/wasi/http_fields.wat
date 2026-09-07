;; L-HTTP-FIELDS: wasi:http/types@0.3.0 fields + request/response get-headers.
;; Guest: constructor fields → append x-p3/flds → has/get; request.get-headers is
;; immutable (append → header-error.immutable). Harness 1.
(component
  (import "wasi:http/types@0.3.0" (instance $types
    (export "fields" (type $fields (sub resource)))
    (export "request" (type $request (sub resource)))
    (export "response" (type $response (sub resource)))
    (type $header-error-def (variant
      (case "invalid-syntax")
      (case "forbidden")
      (case "immutable")
    ))
    (export "header-error" (type $header-error (eq $header-error-def)))
    (type $field-value (list u8))
    (type $value-list (list $field-value))
    (type $hdr-result (result (error $header-error)))
    (type $borrow-fields (borrow $fields))
    (type $borrow-req (borrow $request))
    (type $borrow-resp (borrow $response))
    (export "[constructor]fields" (func (result (own $fields))))
    (export "[constructor]request" (func (result (own $request))))
    (export "[constructor]response" (func (result (own $response))))
    (export "[method]fields.get"
      (func (param "self" $borrow-fields) (param "name" string) (result $value-list)))
    (export "[method]fields.has"
      (func (param "self" $borrow-fields) (param "name" string) (result bool)))
    (export "[method]fields.append"
      (func (param "self" $borrow-fields) (param "name" string) (param "value" $field-value) (result $hdr-result)))
    (export "[method]request.get-headers"
      (func (param "self" $borrow-req) (result (own $fields))))
    (export "[method]response.get-headers"
      (func (param "self" $borrow-resp) (result (own $fields))))
  ))
  (alias export $types "fields" (type $fields))
  (alias export $types "request" (type $request))
  (alias export $types "response" (type $response))
  (alias export $types "[constructor]fields" (func $fields-ctor))
  (alias export $types "[constructor]request" (func $request-ctor))
  (alias export $types "[constructor]response" (func $response-ctor))
  (alias export $types "[method]fields.get" (func $fields-get))
  (alias export $types "[method]fields.has" (func $fields-has))
  (alias export $types "[method]fields.append" (func $fields-append))
  (alias export $types "[method]request.get-headers" (func $req-headers))
  (alias export $types "[method]response.get-headers" (func $resp-headers))

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
    (data (i32.const 16) "x-p3flds")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "fields-ctor" (func $fields-ctor (result i32)))
    (import "" "request-ctor" (func $request-ctor (result i32)))
    (import "" "response-ctor" (func $response-ctor (result i32)))
    (import "" "fields-get" (func $fields-get (param i32 i32 i32 i32)))
    (import "" "fields-has" (func $fields-has (param i32 i32 i32) (result i32)))
    (import "" "fields-append" (func $fields-append (param i32 i32 i32 i32 i32 i32)))
    (import "" "req-headers" (func $req-headers (param i32) (result i32)))
    (import "" "resp-headers" (func $resp-headers (param i32) (result i32)))

    (func (export "run") (result i32)
      (local $f i32)
      (local $req i32)
      (local $resp i32)
      (local $hdrs i32)
      (local $ptr i32)
      (local $n i32)
      (local $vptr i32)
      (local $vlen i32)

      (local.set $f (call $fields-ctor))
      (call $fields-append
        (local.get $f)
        (i32.const 16) (i32.const 4)
        (i32.const 20) (i32.const 4)
        (i32.const 80))
      (if (i32.ne (i32.load8_u (i32.const 80)) (i32.const 0))
        (then (return (i32.const 0))))
      (if (i32.eqz (call $fields-has (local.get $f) (i32.const 16) (i32.const 4)))
        (then (return (i32.const 0))))
      (call $fields-get (local.get $f) (i32.const 16) (i32.const 4) (i32.const 80))
      (local.set $ptr (i32.load (i32.const 80)))
      (local.set $n (i32.load (i32.const 84)))
      (if (i32.ne (local.get $n) (i32.const 1))
        (then (return (i32.const 0))))
      (local.set $vptr (i32.load (local.get $ptr)))
      (local.set $vlen (i32.load (i32.add (local.get $ptr) (i32.const 4))))
      (if (i32.ne (local.get $vlen) (i32.const 4))
        (then (return (i32.const 0))))
      (if (i32.ne (i32.load (local.get $vptr)) (i32.load (i32.const 20)))
        (then (return (i32.const 0))))

      (local.set $req (call $request-ctor))
      (local.set $hdrs (call $req-headers (local.get $req)))
      (if (call $fields-has (local.get $hdrs) (i32.const 16) (i32.const 4))
        (then (return (i32.const 0))))
      (call $fields-append
        (local.get $hdrs)
        (i32.const 16) (i32.const 4)
        (i32.const 20) (i32.const 4)
        (i32.const 80))
      (if (i32.ne (i32.load8_u (i32.const 80)) (i32.const 1))
        (then (return (i32.const 0))))

      (local.set $resp (call $response-ctor))
      (local.set $hdrs (call $resp-headers (local.get $resp)))
      (if (call $fields-has (local.get $hdrs) (i32.const 16) (i32.const 4))
        (then (return (i32.const 0))))

      (i32.const 1)
    )
  )

  (core func $fields_ctor_lower (canon lower (func $fields-ctor)))
  (core func $request_ctor_lower (canon lower (func $request-ctor)))
  (core func $response_ctor_lower (canon lower (func $response-ctor)))
  (core func $get_lower
    (canon lower (func $fields-get)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $has_lower
    (canon lower (func $fields-has)
      (memory $libc "mem")))
  (core func $append_lower
    (canon lower (func $fields-append)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $req_hdr_lower (canon lower (func $req-headers)))
  (core func $resp_hdr_lower (canon lower (func $resp-headers)))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "fields-ctor" (func $fields_ctor_lower))
      (export "request-ctor" (func $request_ctor_lower))
      (export "response-ctor" (func $response_ctor_lower))
      (export "fields-get" (func $get_lower))
      (export "fields-has" (func $has_lower))
      (export "fields-append" (func $append_lower))
      (export "req-headers" (func $req_hdr_lower))
      (export "resp-headers" (func $resp_hdr_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
