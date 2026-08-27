;; P010-GFXH: wasi-gfx:surface@0.2.0 on-frame CM stream (guest pull).
;; Host produces one frame-event on a helper thread named GpuThread, then
;; guest stream.read. Not the product present loop (P010-GFXL). No JS callback.
(component
  (import "wasi-gfx:surface/surface@0.2.0" (instance $surf
    (export "surface" (type $surface (sub resource)))
    (type $create-desc-def (record
      (field "height" (option u32))
      (field "width" (option u32))))
    (export "create-desc" (type $create-desc (eq $create-desc-def)))
    (type $frame-event-def (record (field "nothing" bool)))
    (export "frame-event" (type $frame-event (eq $frame-event-def)))
    (type $st (stream $frame-event))
    (type $borrow-surf (borrow $surface))
    (export "[constructor]surface"
      (func (param "desc" $create-desc) (result (own $surface))))
    (export "[method]surface.on-frame"
      (func (param "self" $borrow-surf) (result $st)))
  ))
  (alias export $surf "surface" (type $surface))
  (alias export $surf "frame-event" (type $frame-event))
  (alias export $surf "[constructor]surface" (func $ctor))
  (alias export $surf "[method]surface.on-frame" (func $on-frame))
  (type $st (stream $frame-event))

  (core module $libc
    (memory (export "mem") 1)
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "ctor" (func $ctor (param i32 i32 i32 i32) (result i32)))
    (import "" "on-frame" (func $on-frame (param i32) (result i32)))
    (import "" "stream.read" (func $stream.read (param i32 i32 i32) (result i32)))
    (import "" "stream.drop-readable" (func $stream.drop-readable (param i32)))

    (func (export "run") (result i32)
      (local $surf i32)
      (local $s i32)
      (local $status i32)
      (local $n i32)

      ;; create-desc: two option<u32> none (flattened disc,payload × 2)
      (local.set $surf (call $ctor
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)))
      (local.set $s (call $on-frame (local.get $surf)))

      (local.set $status (call $stream.read (local.get $s) (i32.const 32) (i32.const 1)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (local.set $n (i32.shr_u (local.get $status) (i32.const 4)))
      (if (i32.ne (local.get $n) (i32.const 1))
        (then unreachable))
      (if (i32.ne (i32.load8_u (i32.const 32)) (i32.const 1))
        (then unreachable))
      (call $stream.drop-readable (local.get $s))
      (i32.const 1)
    )
  )

  (core func $ctor_lower (canon lower (func $ctor)))
  (core func $on_frame_lower (canon lower (func $on-frame)))
  (core func $stream.read (canon stream.read $st async (memory $libc "mem")))
  (core func $stream.drop-readable (canon stream.drop-readable $st))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "ctor" (func $ctor_lower))
      (export "on-frame" (func $on_frame_lower))
      (export "stream.read" (func $stream.read))
      (export "stream.drop-readable" (func $stream.drop-readable))
    ))
  ))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
