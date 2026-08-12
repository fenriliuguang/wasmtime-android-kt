;; P3-PRIM-3/5 smoke: guest stream.write → host StreamConsumer (write-direction flip).
;; Host import `take` pipes StreamReader and returns future<u32> (byte count).
;; Guest: stream.new → take(readable) → write "P3WR" → drop-writable → future.read.
(component
  (type $st (stream u8))
  (type $ft (future u32))
  (import "take" (func $take (param "s" $st) (result $ft)))

  (core module $libc
    (memory (export "mem") 1)
    (data (i32.const 16) "P3WR")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.new" (func $stream.new (result i64)))
    (import "" "stream.write" (func $stream.write (param i32 i32 i32) (result i32)))
    (import "" "stream.drop-writable" (func $stream.drop-writable (param i32)))
    (import "" "future.read" (func $future.read (param i32 i32) (result i32)))
    (import "" "future.drop-readable" (func $future.drop-readable (param i32)))
    (import "" "take" (func $take (param i32) (result i32)))

    (func (export "run") (result i32)
      (local $pair i64)
      (local $r i32)
      (local $w i32)
      (local $fut i32)
      (local $status i32)

      (local.set $pair (call $stream.new))
      (local.set $r (i32.wrap_i64 (local.get $pair)))
      (local.set $w (i32.wrap_i64 (i64.shr_u (local.get $pair) (i64.const 32))))

      (local.set $fut (call $take (local.get $r)))

      ;; Write 4 bytes; host consumer already piped so this should complete.
      (local.set $status (call $stream.write (local.get $w) (i32.const 16) (i32.const 4)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))

      (call $stream.drop-writable (local.get $w))

      ;; future.read(handle, ptr) — payload u32 at mem[0]
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
  (core func $take_lower (canon lower (func $take) (memory $libc "mem")))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.new" (func $stream.new))
      (export "stream.write" (func $stream.write))
      (export "stream.drop-writable" (func $stream.drop-writable))
      (export "future.read" (func $future.read))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "take" (func $take_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
