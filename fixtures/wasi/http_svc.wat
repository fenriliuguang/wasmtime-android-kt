;; L-HTTP-SVC: remaining incoming-handler types for guest handle.
;; Official: get-method / get-path-with-query / get-scheme / get-authority
;; and response set-status-code. Not a listen HTTP server. Product path:
;; no [constructor]request/response; host supplies the request.
;;
;; handle: GET + path "/svc" + some authority + some scheme → 201
;;         otherwise 405 (method) / 404 (path) / 400 (authority or scheme).
;; Root `run` still returns 200 from empty response.new (harness).
(component
  (import "wasi:http/types@0.3.0" (instance $types
    (export "request" (type $request (sub resource)))
    (export "response" (type $response (sub resource)))
    (type $method-def (variant
      (case "get")
      (case "head")
      (case "post")
      (case "put")
      (case "delete")
      (case "connect")
      (case "options")
      (case "trace")
      (case "patch")
      (case "other" string)
    ))
    (export "method" (type $method (eq $method-def)))
    (type $scheme-def (variant
      (case "HTTP")
      (case "HTTPS")
      (case "other" string)
    ))
    (export "scheme" (type $scheme (eq $scheme-def)))
    (type $dns-payload (record (field "rcode" (option string)) (field "info-code" (option u16))))
    (export "dns-error-payload" (type $dns-ex (eq $dns-payload)))
    (type $tls-alert (record (field "alert-id" (option u8)) (field "alert-message" (option string))))
    (export "tls-alert-received-payload" (type $tls-ex (eq $tls-alert)))
    (type $field-size (record (field "field-name" (option string)) (field "field-size" (option u32))))
    (export "field-size-payload" (type $field-ex (eq $field-size)))
    (type $error-code-def (variant
      (case "DNS-timeout")
      (case "DNS-error" $dns-ex)
      (case "destination-not-found")
      (case "destination-unavailable")
      (case "destination-IP-prohibited")
      (case "destination-IP-unroutable")
      (case "connection-refused")
      (case "connection-terminated")
      (case "connection-timeout")
      (case "connection-read-timeout")
      (case "connection-write-timeout")
      (case "connection-limit-reached")
      (case "TLS-protocol-error")
      (case "TLS-certificate-error")
      (case "TLS-alert-received" $tls-ex)
      (case "HTTP-request-denied")
      (case "HTTP-request-length-required")
      (case "HTTP-request-body-size" (option u64))
      (case "HTTP-request-method-invalid")
      (case "HTTP-request-URI-invalid")
      (case "HTTP-request-URI-too-long")
      (case "HTTP-request-header-section-size" (option u32))
      (case "HTTP-request-header-size" (option $field-ex))
      (case "HTTP-request-trailer-section-size" (option u32))
      (case "HTTP-request-trailer-size" $field-ex)
      (case "HTTP-response-incomplete")
      (case "HTTP-response-header-section-size" (option u32))
      (case "HTTP-response-header-size" $field-ex)
      (case "HTTP-response-body-size" (option u64))
      (case "HTTP-response-trailer-section-size" (option u32))
      (case "HTTP-response-trailer-size" $field-ex)
      (case "HTTP-response-transfer-coding" (option string))
      (case "HTTP-response-content-coding" (option string))
      (case "HTTP-response-timeout")
      (case "HTTP-upgrade-failed")
      (case "HTTP-protocol-error")
      (case "loop-detected")
      (case "configuration-error")
      (case "internal-error" (option string))
    ))
    (export "error-code" (type $error-code (eq $error-code-def)))
    (type $io-result (result (error $error-code)))
    (type $st (stream u8))
    (type $ft (future $io-result))
    (type $new-ret (tuple (own $response) $ft))
    (type $borrow-req (borrow $request))
    (type $borrow-resp (borrow $response))
    (type $opt-str (option string))
    (type $opt-scheme (option $scheme))
    (type $unit-result (result))
    (export "[static]response.new"
      (func (param "contents" $st) (result $new-ret)))
    (export "[method]response.status-code"
      (func (param "self" $borrow-resp) (result u16)))
    (export "[method]response.set-status-code"
      (func (param "self" $borrow-resp) (param "status-code" u16) (result $unit-result)))
    (export "[method]request.get-method"
      (func (param "self" $borrow-req) (result $method)))
    (export "[method]request.get-path-with-query"
      (func (param "self" $borrow-req) (result $opt-str)))
    (export "[method]request.get-scheme"
      (func (param "self" $borrow-req) (result $opt-scheme)))
    (export "[method]request.get-authority"
      (func (param "self" $borrow-req) (result $opt-str)))
  ))
  (alias export $types "request" (type $request))
  (alias export $types "response" (type $response))
  (alias export $types "error-code" (type $error-code))
  (alias export $types "[static]response.new" (func $response-new))
  (alias export $types "[method]response.status-code" (func $status-code))
  (alias export $types "[method]response.set-status-code" (func $set-status-code))
  (alias export $types "[method]request.get-method" (func $get-method))
  (alias export $types "[method]request.get-path-with-query" (func $get-path))
  (alias export $types "[method]request.get-scheme" (func $get-scheme))
  (alias export $types "[method]request.get-authority" (func $get-authority))
  (type $io-result (result (error $error-code)))
  (type $st (stream u8))
  (type $ft (future $io-result))
  (type $handle-result (result (own $response) (error $error-code)))

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
    (data (i32.const 32) "/svc")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.new" (func $stream.new (result i64)))
    (import "" "stream.drop-writable" (func $stream.drop-writable (param i32)))
    (import "" "future.drop-readable" (func $future.drop-readable (param i32)))
    (import "" "response-new" (func $response-new (param i32 i32)))
    (import "" "status-code" (func $status-code (param i32) (result i32)))
    (import "" "set-status-code" (func $set-status-code (param i32 i32) (result i32)))
    (import "" "get-method" (func $get-method (param i32 i32)))
    (import "" "get-path" (func $get-path (param i32 i32)))
    (import "" "get-scheme" (func $get-scheme (param i32 i32)))
    (import "" "get-authority" (func $get-authority (param i32 i32)))

    (func $bytes-eq (param $a i32) (param $b i32) (param $n i32) (result i32)
      (local $i i32)
      (loop $l
        (if (i32.eq (local.get $i) (local.get $n))
          (then (return (i32.const 1))))
        (if (i32.ne
              (i32.load8_u (i32.add (local.get $a) (local.get $i)))
              (i32.load8_u (i32.add (local.get $b) (local.get $i))))
          (then (return (i32.const 0))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $l)
      )
      (i32.const 0)
    )

    (func $empty-response (result i32)
      (local $pair i64)
      (local $r i32)
      (local $w i32)
      (local $fut i32)
      (local.set $pair (call $stream.new))
      (local.set $r (i32.wrap_i64 (local.get $pair)))
      (local.set $w (i32.wrap_i64 (i64.shr_u (local.get $pair) (i64.const 32))))
      (call $response-new (local.get $r) (i32.const 80))
      (call $stream.drop-writable (local.get $w))
      (local.set $fut (i32.load (i32.const 84)))
      (call $future.drop-readable (local.get $fut))
      (i32.load (i32.const 80))
    )

    (func (export "handle") (param $req i32) (result i32)
      (local $st i32)
      (local $resp i32)
      (local.set $st (i32.const 201))

      (call $get-method (local.get $req) (i32.const 128))
      (if (i32.ne (i32.load8_u (i32.const 128)) (i32.const 0))
        (then (local.set $st (i32.const 405))))

      (if (i32.eq (local.get $st) (i32.const 201))
        (then
          (call $get-path (local.get $req) (i32.const 144))
          (if (i32.eqz (i32.load8_u (i32.const 144)))
            (then (local.set $st (i32.const 404)))
            (else
              (if (i32.or
                    (i32.ne (i32.load (i32.const 152)) (i32.const 4))
                    (i32.eqz (call $bytes-eq
                      (i32.load (i32.const 148)) (i32.const 32) (i32.const 4))))
                (then (local.set $st (i32.const 404))))))))

      (if (i32.eq (local.get $st) (i32.const 201))
        (then
          (call $get-authority (local.get $req) (i32.const 160))
          (if (i32.eqz (i32.load8_u (i32.const 160)))
            (then (local.set $st (i32.const 400))))))

      (if (i32.eq (local.get $st) (i32.const 201))
        (then
          (call $get-scheme (local.get $req) (i32.const 176))
          (if (i32.eqz (i32.load8_u (i32.const 176)))
            (then (local.set $st (i32.const 400))))))

      (local.set $resp (call $empty-response))
      (drop (call $set-status-code (local.get $resp) (local.get $st)))
      (i32.store8 (i32.const 16) (i32.const 0))
      (i32.store (i32.const 24) (local.get $resp))
      (i32.const 16)
    )

    (func (export "run") (result i32)
      (call $status-code (call $empty-response))
    )
  )

  (core func $stream.new (canon stream.new $st))
  (core func $stream.drop-writable (canon stream.drop-writable $st))
  (core func $future.drop-readable (canon future.drop-readable $ft))
  (core func $response_new_lower
    (canon lower (func $response-new)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $status_lower (canon lower (func $status-code)))
  (core func $set_status_lower
    (canon lower (func $set-status-code)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $get_method_lower
    (canon lower (func $get-method)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $get_path_lower
    (canon lower (func $get-path)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $get_scheme_lower
    (canon lower (func $get-scheme)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $get_authority_lower
    (canon lower (func $get-authority)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.new" (func $stream.new))
      (export "stream.drop-writable" (func $stream.drop-writable))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "response-new" (func $response_new_lower))
      (export "status-code" (func $status_lower))
      (export "set-status-code" (func $set_status_lower))
      (export "get-method" (func $get_method_lower))
      (export "get-path" (func $get_path_lower))
      (export "get-scheme" (func $get_scheme_lower))
      (export "get-authority" (func $get_authority_lower))
    ))
  ))

  (func $handle-lift (param "request" (own $request)) (result $handle-result)
    (canon lift (core func $i "handle")
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (instance $handler
    (export "handle" (func $handle-lift)))
  (export "wasi:http/incoming-handler@0.3.0" (instance $handler))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
