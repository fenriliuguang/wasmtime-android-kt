;; L-CMD-ENV: wasi:cli/environment@0.3.0 get-environment / get-arguments.
;; Guest: arguments must be empty; environment must contain TMPDIR=/tmp/p3env.
;; Host supplies TMPDIR only (Android: empty or documented TMPDIR). Not full env.
(component
  (import "wasi:cli/environment@0.3.0" (instance $environment
    (export "get-environment"
      (func (result (list (tuple string string)))))
    (export "get-arguments"
      (func (result (list string))))
  ))
  (alias export $environment "get-environment" (func $get-environment))
  (alias export $environment "get-arguments" (func $get-arguments))

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
    (data (i32.const 16) "TMPDIR")
    (data (i32.const 24) "/tmp/p3env")
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-environment" (func $get-environment (param i32)))
    (import "" "get-arguments" (func $get-arguments (param i32)))

    (func $eq (param $a i32) (param $b i32) (param $n i32) (result i32)
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

    (func (export "run") (result i32)
      (local $ptr i32)
      (local $n i32)
      (local $i i32)
      (local $kptr i32)
      (local $klen i32)
      (local $vptr i32)
      (local $vlen i32)
      (local $off i32)

      ;; arguments: list at 192 {ptr, len}; must be empty.
      (call $get-arguments (i32.const 192))
      (if (i32.ne (i32.load (i32.const 196)) (i32.const 0))
        (then unreachable))

      ;; environment: list at 208 {ptr, len}; 16-byte tuple<string,string> elems.
      (call $get-environment (i32.const 208))
      (local.set $ptr (i32.load (i32.const 208)))
      (local.set $n (i32.load (i32.const 212)))
      (loop $l
        (if (i32.eq (local.get $i) (local.get $n))
          (then (return (i32.const 0))))
        (local.set $off (i32.mul (local.get $i) (i32.const 16)))
        (local.set $kptr (i32.load (i32.add (local.get $ptr) (local.get $off))))
        (local.set $klen
          (i32.load (i32.add (local.get $ptr) (i32.add (local.get $off) (i32.const 4)))))
        (local.set $vptr
          (i32.load (i32.add (local.get $ptr) (i32.add (local.get $off) (i32.const 8)))))
        (local.set $vlen
          (i32.load (i32.add (local.get $ptr) (i32.add (local.get $off) (i32.const 12)))))
        (if (i32.and
              (i32.and
                (i32.eq (local.get $klen) (i32.const 6))
                (call $eq (local.get $kptr) (i32.const 16) (i32.const 6)))
              (i32.and
                (i32.eq (local.get $vlen) (i32.const 10))
                (call $eq (local.get $vptr) (i32.const 24) (i32.const 10))))
          (then (return (i32.const 1))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $l)
      )
      (i32.const 0)
    )
  )

  (core func $get_env_lower
    (canon lower (func $get-environment)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $get_args_lower
    (canon lower (func $get-arguments)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "get-environment" (func $get_env_lower))
      (export "get-arguments" (func $get_args_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
