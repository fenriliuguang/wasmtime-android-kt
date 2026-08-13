;; WASI 0.3 package smoke: wasi:cli/stdin@0.3.0#read-via-stream
;; Transitional signature: func() -> stream<u8> (host produces "IN\n").
;; Official WIT is tuple<stream<u8>, future<result<_, error-code>>>; tuple/result deferred.
;; Guest: read-via-stream → stream.read → return nbytes (status >> 4).
(component
  (type $st (stream u8))
  (import "wasi:cli/stdin@0.3.0" (instance $stdin
    (export "read-via-stream" (func (result $st)))
  ))
  (alias export $stdin "read-via-stream" (func $read-via-stream))

  (core module $libc (memory (export "mem") 1))
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.read" (func $stream.read (param i32 i32 i32) (result i32)))
    (import "" "read-via-stream" (func $read-via-stream (result i32)))

    (func (export "run") (result i32)
      (local $s i32)
      (local $status i32)

      (local.set $s (call $read-via-stream))
      (local.set $status (call $stream.read (local.get $s) (i32.const 0) (i32.const 16)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))

      ;; Packed stream.read: (nbytes << 4) | status — return nbytes.
      (i32.shr_u (local.get $status) (i32.const 4))
    )
  )

  (core func $stream.read (canon stream.read $st async (memory $libc "mem")))
  (core func $read_lower (canon lower (func $read-via-stream) (memory $libc "mem")))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.read" (func $stream.read))
      (export "read-via-stream" (func $read_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
