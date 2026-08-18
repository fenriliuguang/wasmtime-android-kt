;; S6+: get-pass + [method]gpu-render-pass-encoder.set-scissor-rect
;; WIT: set-scissor-rect: func(x, y, width, height: u32)
;; Guest passes 0,0,1,1; run returns harness 1. L2 unused (no new JNI).
;; get-pass is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $set-ty (func
      (param "self" $borrow-pass)
      (param "x" u32)
      (param "y" u32)
      (param "width" u32)
      (param "height" u32)))
    (export "[method]gpu-render-pass-encoder.set-scissor-rect" (func (type $set-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.set-scissor-rect" (func $set-sc))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $ss_lower (canon lower (func $set-sc)))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "set-sc" (func $set-sc (param i32 i32 i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-sc
        (call $get-pass)
        (i32.const 0) (i32.const 0)
        (i32.const 1) (i32.const 1))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "set-sc" (func $ss_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
