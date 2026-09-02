;; GFX-PIN input: wasi-gfx:surface@0.2.0 on-pointer-down stream.read.
;; Host posts one pointer sample before run. Guest reads x=12.5 y=34.0.
(component
  (import "wasi-gfx:surface/surface@0.2.0" (instance $surf
    (export "surface" (type $surface (sub resource)))
    (type $create-desc-def (record
      (field "height" (option u32))
      (field "width" (option u32))))
    (export "create-desc" (type $create-desc (eq $create-desc-def)))
    (type $pointer-event-def (record (field "x" f64) (field "y" f64)))
    (export "pointer-event" (type $pointer-event (eq $pointer-event-def)))
    (type $st (stream $pointer-event))
    (type $borrow-surf (borrow $surface))
    (export "[constructor]surface"
      (func (param "desc" $create-desc) (result (own $surface))))
    (export "[method]surface.on-pointer-down"
      (func (param "self" $borrow-surf) (result $st)))
  ))
  (alias export $surf "surface" (type $surface))
  (alias export $surf "pointer-event" (type $pointer-event))
  (alias export $surf "[constructor]surface" (func $ctor))
  (alias export $surf "[method]surface.on-pointer-down" (func $on-down))
  (type $st (stream $pointer-event))

  (core module $libc
    (memory (export "mem") 1)
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "ctor" (func $ctor (param i32 i32 i32 i32) (result i32)))
    (import "" "on-down" (func $on-down (param i32) (result i32)))
    (import "" "stream.read" (func $stream.read (param i32 i32 i32) (result i32)))
    (import "" "stream.drop-readable" (func $stream.drop-readable (param i32)))

    (func (export "run") (result i32)
      (local $surf i32)
      (local $s i32)
      (local $status i32)
      (local $n i32)

      (local.set $surf (call $ctor
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)))
      (local.set $s (call $on-down (local.get $surf)))
      (local.set $status (call $stream.read (local.get $s) (i32.const 32) (i32.const 1)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then (return (i32.const 2))))
      (local.set $n (i32.shr_u (local.get $status) (i32.const 4)))
      (if (i32.ne (local.get $n) (i32.const 1))
        (then (return (i32.const 3))))
      (if (f64.ne (f64.load (i32.const 32)) (f64.const 12.5))
        (then (return (i32.const 4))))
      (if (f64.ne (f64.load (i32.const 40)) (f64.const 34.0))
        (then (return (i32.const 5))))
      (call $stream.drop-readable (local.get $s))
      (i32.const 1)
    )
  )

  (core func $ctor_lower (canon lower (func $ctor)))
  (core func $on_down_lower (canon lower (func $on-down)))
  (core func $stream.read (canon stream.read $st async (memory $libc "mem")))
  (core func $stream.drop-readable (canon stream.drop-readable $st))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "ctor" (func $ctor_lower))
      (export "on-down" (func $on_down_lower))
      (export "stream.read" (func $stream.read))
      (export "stream.drop-readable" (func $stream.drop-readable))
    ))
  ))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
