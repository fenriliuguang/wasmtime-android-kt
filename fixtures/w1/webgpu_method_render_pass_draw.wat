;; S6+: get-pass + [method]gpu-render-pass-encoder.draw
;; WIT: draw: func(vertex-count: u32, instance-count: option<u32>,
;;      first-vertex: option<u32>, first-instance: option<u32>)
;; Guest passes vertex-count=3, other fields none; run returns harness 1.
;; L2 described JNI forwards pass rep + vertex-count (options none → 1/0/0).
;; get-pass is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $opt-u32 (option u32))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $draw-ty (func
      (param "self" $borrow-pass)
      (param "vertex-count" u32)
      (param "instance-count" $opt-u32)
      (param "first-vertex" $opt-u32)
      (param "first-instance" $opt-u32)))
    (export "[method]gpu-render-pass-encoder.draw" (func (type $draw-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.draw" (func $draw))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $d_lower (canon lower (func $draw)))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "draw" (func $draw (param i32 i32 i32 i32 i32 i32 i32 i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local.set $pass (call $get-pass))
      (call $draw
        (local.get $pass)
        (i32.const 3)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "draw" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
