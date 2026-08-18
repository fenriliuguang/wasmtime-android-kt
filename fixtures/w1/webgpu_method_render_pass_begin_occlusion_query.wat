;; S6+: get-pass + [method]gpu-render-pass-encoder.begin-occlusion-query
;; WIT: begin-occlusion-query: func(query-index: u32)
;; Guest passes query-index=0; run returns harness 1. L2 unused.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $begin-ty (func
      (param "self" $borrow-pass)
      (param "query-index" u32)))
    (export "[method]gpu-render-pass-encoder.begin-occlusion-query" (func (type $begin-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.begin-occlusion-query" (func $begin))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $b_lower (canon lower (func $begin)))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "begin" (func $begin (param i32 i32)))
    (func (export "run") (result i32)
      (call $begin (call $get-pass) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "begin" (func $b_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
