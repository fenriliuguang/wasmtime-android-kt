;; WASI 0.3: wasi:random/random@0.3.0#get-random-bytes
;; Guest calls get-random-bytes(8), packs first 8 bytes LE into u64, exports run.
(component
  (import "wasi:random/random@0.3.0" (instance $random
    (export "get-random-bytes" (func (param "len" u64) (result (list u8))))
  ))
  (alias export $random "get-random-bytes" (func $get-random-bytes))

  (core module $libc
    (memory (export "mem") 1)
    (global $last (mut i32) (i32.const 32))
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
  )
  (core instance $libc (instantiate $libc))

  (core func $grb_lower
    (canon lower (func $get-random-bytes)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-random-bytes" (func $grb (param i64 i32)))
    (func (export "run") (result i64)
      (local $retptr i32)
      (local $ptr i32)
      (local.set $retptr (i32.const 0))
      (call $grb (i64.const 8) (local.get $retptr))
      (local.set $ptr (i32.load (local.get $retptr)))
      (i64.load (local.get $ptr))
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "get-random-bytes" (func $grb_lower))
    ))
  ))
  (func (export "run") (result u64)
    (canon lift (core func $i "run")))
)
