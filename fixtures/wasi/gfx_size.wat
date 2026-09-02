;; GFX-SIZE: wasi-gfx:surface@0.2.0 height / width / request-set-size / on-resize.
;; Host binds a window (64x48) before run. Guest checks getters, requests 96x80,
;; then stream.read one resize-event. No JS callback.
(component
  (import "wasi-gfx:surface/surface@0.2.0" (instance $surf
    (export "surface" (type $surface (sub resource)))
    (type $create-desc-def (record
      (field "height" (option u32))
      (field "width" (option u32))))
    (export "create-desc" (type $create-desc (eq $create-desc-def)))
    (type $resize-event-def (record (field "height" u32) (field "width" u32)))
    (export "resize-event" (type $resize-event (eq $resize-event-def)))
    (type $st (stream $resize-event))
    (type $borrow-surf (borrow $surface))
    (export "[constructor]surface"
      (func (param "desc" $create-desc) (result (own $surface))))
    (export "[method]surface.height"
      (func (param "self" $borrow-surf) (result u32)))
    (export "[method]surface.width"
      (func (param "self" $borrow-surf) (result u32)))
    (export "[method]surface.request-set-size"
      (func (param "self" $borrow-surf) (param "height" (option u32)) (param "width" (option u32))))
    (export "[method]surface.on-resize"
      (func (param "self" $borrow-surf) (result $st)))
  ))
  (alias export $surf "surface" (type $surface))
  (alias export $surf "resize-event" (type $resize-event))
  (alias export $surf "[constructor]surface" (func $ctor))
  (alias export $surf "[method]surface.height" (func $height))
  (alias export $surf "[method]surface.width" (func $width))
  (alias export $surf "[method]surface.request-set-size" (func $request))
  (alias export $surf "[method]surface.on-resize" (func $on-resize))
  (type $st (stream $resize-event))

  (core module $libc
    (memory (export "mem") 1)
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "ctor" (func $ctor (param i32 i32 i32 i32) (result i32)))
    (import "" "height" (func $height (param i32) (result i32)))
    (import "" "width" (func $width (param i32) (result i32)))
    (import "" "request" (func $request (param i32 i32 i32 i32 i32)))
    (import "" "on-resize" (func $on-resize (param i32) (result i32)))
    (import "" "stream.read" (func $stream.read (param i32 i32 i32) (result i32)))
    (import "" "stream.drop-readable" (func $stream.drop-readable (param i32)))

    (func (export "run") (result i32)
      (local $surf i32)
      (local $s i32)
      (local $status i32)
      (local $n i32)

      (local.set $surf (call $ctor
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)))
      (if (i32.ne (call $height (local.get $surf)) (i32.const 48))
        (then (return (i32.const 2))))
      (if (i32.ne (call $width (local.get $surf)) (i32.const 64))
        (then (return (i32.const 3))))
      ;; option some(96), some(80)
      (call $request (local.get $surf)
        (i32.const 1) (i32.const 96) (i32.const 1) (i32.const 80))
      (if (i32.ne (call $height (local.get $surf)) (i32.const 96))
        (then (return (i32.const 4))))
      (if (i32.ne (call $width (local.get $surf)) (i32.const 80))
        (then (return (i32.const 5))))
      (local.set $s (call $on-resize (local.get $surf)))
      (local.set $status (call $stream.read (local.get $s) (i32.const 32) (i32.const 1)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then (return (i32.const 6))))
      (local.set $n (i32.shr_u (local.get $status) (i32.const 4)))
      (if (i32.ne (local.get $n) (i32.const 1))
        (then (return (i32.const 7))))
      (if (i32.ne (i32.load (i32.const 32)) (i32.const 96))
        (then (return (i32.const 8))))
      (if (i32.ne (i32.load (i32.const 36)) (i32.const 80))
        (then (return (i32.const 9))))
      (call $stream.drop-readable (local.get $s))
      (i32.const 1)
    )
  )

  (core func $ctor_lower (canon lower (func $ctor)))
  (core func $height_lower (canon lower (func $height)))
  (core func $width_lower (canon lower (func $width)))
  (core func $request_lower (canon lower (func $request)))
  (core func $on_resize_lower (canon lower (func $on-resize)))
  (core func $stream.read (canon stream.read $st async (memory $libc "mem")))
  (core func $stream.drop-readable (canon stream.drop-readable $st))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "ctor" (func $ctor_lower))
      (export "height" (func $height_lower))
      (export "width" (func $width_lower))
      (export "request" (func $request_lower))
      (export "on-resize" (func $on_resize_lower))
      (export "stream.read" (func $stream.read))
      (export "stream.drop-readable" (func $stream.drop-readable))
    ))
  ))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
