;; W1 multi-chunk + backpressure: guest stream.write three 4-byte chunks.
;; Host import `take-chunks` pipes a StreamConsumer that takes 2 bytes/poll.
;; Each write may complete with a partial count; guest retries remaining.
;; Guest: stream.new → take-chunks(readable) → write "P3C1" "P3C2" "P3C3"
;;        → drop-writable → future.read. Expected: 12.
;; Not a second copy of the 4-byte P3WR / P3ST smokes.
(component
  (type $st (stream u8))
  (type $ft (future u32))
  (import "take-chunks" (func $take-chunks (param "s" $st) (result $ft)))

  (core module $libc
    (memory (export "mem") 1)
    (data (i32.const 16) "P3C1P3C2P3C3")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.new" (func $stream.new (result i64)))
    (import "" "stream.write" (func $stream.write (param i32 i32 i32) (result i32)))
    (import "" "stream.drop-writable" (func $stream.drop-writable (param i32)))
    (import "" "future.read" (func $future.read (param i32 i32) (result i32)))
    (import "" "future.drop-readable" (func $future.drop-readable (param i32)))
    (import "" "take-chunks" (func $take-chunks (param i32) (result i32)))

    ;; Retry until `len` bytes are written or the readable side drops.
    ;; Packed result: (nbytes << 4) | status; -1 = BLOCKED (async must not leak).
    (func $write_all (param $w i32) (param $ptr i32) (param $len i32)
      (local $status i32)
      (local $n i32)
      (loop $again
        (if (i32.eqz (local.get $len))
          (then (return)))
        (local.set $status
          (call $stream.write (local.get $w) (local.get $ptr) (local.get $len)))
        (if (i32.eq (local.get $status) (i32.const -1))
          (then unreachable))
        (local.set $n (i32.shr_u (local.get $status) (i32.const 4)))
        (if (i32.eqz (local.get $n))
          (then unreachable))
        (local.set $ptr (i32.add (local.get $ptr) (local.get $n)))
        (local.set $len (i32.sub (local.get $len) (local.get $n)))
        ;; DROPPED (low nibble 1): remaining cannot be written.
        (if (i32.eq (i32.and (local.get $status) (i32.const 0xf)) (i32.const 1))
          (then (return)))
        (br $again)
      )
    )

    (func (export "run") (result i32)
      (local $pair i64)
      (local $r i32)
      (local $w i32)
      (local $fut i32)
      (local $status i32)

      (local.set $pair (call $stream.new))
      (local.set $r (i32.wrap_i64 (local.get $pair)))
      (local.set $w (i32.wrap_i64 (i64.shr_u (local.get $pair) (i64.const 32))))

      (local.set $fut (call $take-chunks (local.get $r)))

      ;; Three 4-byte writes; host takes 2 bytes/poll so each write retries.
      (call $write_all (local.get $w) (i32.const 16) (i32.const 4))
      (call $write_all (local.get $w) (i32.const 20) (i32.const 4))
      (call $write_all (local.get $w) (i32.const 24) (i32.const 4))

      (call $stream.drop-writable (local.get $w))

      (local.set $status (call $future.read (local.get $fut) (i32.const 0)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))

      (i32.load (i32.const 0))
    )
  )

  (core func $stream.new (canon stream.new $st))
  (core func $stream.write (canon stream.write $st async (memory $libc "mem")))
  (core func $stream.drop-writable (canon stream.drop-writable $st))
  (core func $future.read (canon future.read $ft async (memory $libc "mem")))
  (core func $future.drop-readable (canon future.drop-readable $ft))
  (core func $take_chunks_lower (canon lower (func $take-chunks) (memory $libc "mem")))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.new" (func $stream.new))
      (export "stream.write" (func $stream.write))
      (export "stream.drop-writable" (func $stream.drop-writable))
      (export "future.read" (func $future.read))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "take-chunks" (func $take_chunks_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
