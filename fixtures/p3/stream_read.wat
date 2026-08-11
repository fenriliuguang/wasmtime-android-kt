;; P3-PRIM-3 smoke: guest reads host-produced stream<u8> via canon stream.read.
;; Host creates StreamReader (Vec producer) and calls export `read`.
;; Return encoding matches Wasmtime: (nbytes << 4) | status (1 = DROPPED/EOF).
(component
  (core module $libc (memory (export "mem") 1))
  (core instance $libc (instantiate $libc))
  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.read" (func $stream.read (param i32 i32 i32) (result i32)))

    (func (export "read") (param i32 i32) (result i32)
      (call $stream.read (local.get 0) (i32.const 0) (local.get 1))
    )
  )
  (type $s (stream u8))
  (core func $stream.read (canon stream.read $s async (memory $libc "mem")))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.read" (func $stream.read))
    ))
  ))
  (func (export "read") (param "s" (stream u8)) (param "l" u32) (result u32)
    (canon lift (core func $i "read")))
)
