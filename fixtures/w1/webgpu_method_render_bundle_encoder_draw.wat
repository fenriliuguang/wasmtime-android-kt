;; S6+: get-render-bundle-encoder + [method]gpu-render-bundle-encoder.draw
;; WIT: draw: func(vertex-count: u32, instance-count: option<u32>,
;;      first-vertex: option<u32>, first-instance: option<u32>)
;; Guest passes vertex-count=3, other fields none; run returns harness 1. L2 unused.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $opt-u32 (option u32))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $draw-ty (func
      (param "self" $borrow-encoder)
      (param "vertex-count" u32)
      (param "instance-count" $opt-u32)
      (param "first-vertex" $opt-u32)
      (param "first-instance" $opt-u32)))
    (export "[method]gpu-render-bundle-encoder.draw" (func (type $draw-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
  ))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.draw" (func $draw))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $d_lower (canon lower (func $draw)))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "draw" (func $draw (param i32 i32 i32 i32 i32 i32 i32 i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local.set $encoder (call $get-encoder))
      (call $draw
        (local.get $encoder)
        (i32.const 3)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "draw" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
