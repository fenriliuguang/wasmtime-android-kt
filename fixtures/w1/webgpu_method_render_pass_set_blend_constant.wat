;; S6+: get-pass + [method]gpu-render-pass-encoder.set-blend-constant
;; WIT: set-blend-constant: func(color: gpu-color)
;; Guest passes r=g=b=0 a=1; run returns harness 1. L2 unused (no new JNI).
;; get-pass is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $color (record (field "r" f64) (field "g" f64) (field "b" f64) (field "a" f64)))
    (export "gpu-color" (type $gpu-color (eq $color)))
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $set-ty (func
      (param "self" $borrow-pass)
      (param "color" $gpu-color)))
    (export "[method]gpu-render-pass-encoder.set-blend-constant" (func (type $set-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.set-blend-constant" (func $set-bc))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $sbc_lower (canon lower (func $set-bc)))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "set-bc" (func $set-bc (param i32 f64 f64 f64 f64)))
    (func (export "run") (result i32)
      (call $set-bc
        (call $get-pass)
        (f64.const 0) (f64.const 0) (f64.const 0) (f64.const 1))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "set-bc" (func $sbc_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
