;; WASI 0.3 package smoke: wasi:cli/stdin@0.3.0#read-via-stream
;; Official: func() -> tuple<stream<u8>, future<result<_, error-code>>> (ok path).
;; Canon lower stores the tuple in memory (two handles > max flat results).
;; Host produces "IN\n". Guest: read-via-stream → stream.read → future.read (ok) → nbytes.
(component
  (import "wasi:cli/stdin@0.3.0" (instance $stdin
    (type $error-code-def (enum "io" "illegal-byte-sequence" "pipe"))
    (export "error-code" (type $error-code (eq $error-code-def)))
    (type $read-result (result (error $error-code)))
    (type $st (stream u8))
    (type $ft (future $read-result))
    (type $ret (tuple $st $ft))
    (export "read-via-stream" (func (result $ret)))
  ))
  (alias export $stdin "read-via-stream" (func $read-via-stream))
  (alias export $stdin "error-code" (type $error-code))
  (type $read-result (result (error $error-code)))
  (type $st (stream u8))
  (type $ft (future $read-result))

  (core module $libc (memory (export "mem") 1))
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.read" (func $stream.read (param i32 i32 i32) (result i32)))
    (import "" "future.read" (func $future.read (param i32 i32) (result i32)))
    (import "" "future.drop-readable" (func $future.drop-readable (param i32)))
    (import "" "read-via-stream" (func $read-via-stream (param i32)))

    (func (export "run") (result i32)
      (local $s i32)
      (local $fut i32)
      (local $status i32)
      (local $n i32)

      ;; tuple at mem[0]: stream handle, future handle
      (call $read-via-stream (i32.const 0))
      (local.set $s (i32.load (i32.const 0)))
      (local.set $fut (i32.load (i32.const 4)))

      (local.set $status (call $stream.read (local.get $s) (i32.const 16) (i32.const 16)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (local.set $n (i32.shr_u (local.get $status) (i32.const 4)))

      (local.set $status (call $future.read (local.get $fut) (i32.const 32)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))
      (if (i32.ne (i32.load8_u (i32.const 32)) (i32.const 0))
        (then unreachable))

      (local.get $n)
    )
  )

  (core func $stream.read (canon stream.read $st async (memory $libc "mem")))
  (core func $future.read (canon future.read $ft async (memory $libc "mem")))
  (core func $future.drop-readable (canon future.drop-readable $ft))
  (core func $read_lower (canon lower (func $read-via-stream) (memory $libc "mem")))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.read" (func $stream.read))
      (export "future.read" (func $future.read))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "read-via-stream" (func $read_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
