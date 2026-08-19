;; S6+: get-pass + [method]gpu-render-pass-encoder.draw-indexed
;; WIT: draw-indexed: func(index-count, instance-count?, first-index?,
;;      base-vertex?, first-instance?)
;; Guest passes index-count=3, other fields none; run returns harness 1.
;; L2 described JNI forwards pass rep + index-count (options none → 1/0/0/0).
;; get-pass is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $opt-u32 (option u32))
    (type $opt-s32 (option s32))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $draw-ty (func
      (param "self" $borrow-pass)
      (param "index-count" u32)
      (param "instance-count" $opt-u32)
      (param "first-index" $opt-u32)
      (param "base-vertex" $opt-s32)
      (param "first-instance" $opt-u32)))
    (export "[method]gpu-render-pass-encoder.draw-indexed" (func (type $draw-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.draw-indexed" (func $draw-ix))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $d_lower (canon lower (func $draw-ix)))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "draw-ix"
      (func $draw-ix (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)))
    (func (export "run") (result i32)
      (call $draw-ix
        (call $get-pass)
        (i32.const 3)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "draw-ix" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
