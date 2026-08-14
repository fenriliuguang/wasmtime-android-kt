;; WASI 0.3 package smoke: wasi:cli/command-shaped async run
;; Subset: guest exports root `run: async func() -> u32` (0 = ok; official empty result deferred).
;; Imports existing wasi:cli/stdout@0.3.0#write-via-stream (transitional stream<u8> -> future<u32>).
;; Guest: stream.new → write-via-stream(readable) → write "CMD\n" → drop-writable → future.read → return 0.
;; Not a full command world (no fs/sockets/environment/exit/terminal).
(component
  (type $st (stream u8))
  (type $ft (future u32))
  (import "wasi:cli/stdout@0.3.0" (instance $stdout
    (export "write-via-stream" (func (param "data" $st) (result $ft)))
  ))
  (alias export $stdout "write-via-stream" (func $write-via-stream))

  (core module $libc
    (memory (export "mem") 1)
    (data (i32.const 16) "CMD\n")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.new" (func $stream.new (result i64)))
    (import "" "stream.write" (func $stream.write (param i32 i32 i32) (result i32)))
    (import "" "stream.drop-writable" (func $stream.drop-writable (param i32)))
    (import "" "future.read" (func $future.read (param i32 i32) (result i32)))
    (import "" "future.drop-readable" (func $future.drop-readable (param i32)))
    (import "" "write-via-stream" (func $write-via-stream (param i32) (result i32)))

    (func (export "run") (result i32)
      (local $pair i64)
      (local $r i32)
      (local $w i32)
      (local $fut i32)
      (local $status i32)

      (local.set $pair (call $stream.new))
      (local.set $r (i32.wrap_i64 (local.get $pair)))
      (local.set $w (i32.wrap_i64 (i64.shr_u (local.get $pair) (i64.const 32))))

      (local.set $fut (call $write-via-stream (local.get $r)))

      ;; Write 4 bytes ("CMD\n"); host consumer already piped so this should complete.
      (local.set $status (call $stream.write (local.get $w) (i32.const 16) (i32.const 4)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))

      (call $stream.drop-writable (local.get $w))

      ;; future.read(handle, ptr) — payload u32 at mem[0]
      (local.set $status (call $future.read (local.get $fut) (i32.const 0)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))

      ;; Transitional: 0 = ok (official empty result deferred).
      i32.const 0
    )
  )

  (core func $stream.new (canon stream.new $st))
  (core func $stream.write (canon stream.write $st async (memory $libc "mem")))
  (core func $stream.drop-writable (canon stream.drop-writable $st))
  (core func $future.read (canon future.read $ft async (memory $libc "mem")))
  (core func $future.drop-readable (canon future.drop-readable $ft))
  (core func $write_lower (canon lower (func $write-via-stream) (memory $libc "mem")))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.new" (func $stream.new))
      (export "stream.write" (func $stream.write))
      (export "stream.drop-writable" (func $stream.drop-writable))
      (export "future.read" (func $future.read))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "write-via-stream" (func $write_lower))
    ))
  ))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
