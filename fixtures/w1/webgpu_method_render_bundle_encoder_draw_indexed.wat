;; S6+: get-render-bundle-encoder + [method]gpu-render-bundle-encoder.draw-indexed
;; WIT: draw-indexed: func(index-count, instance-count?, first-index?,
;;      base-vertex?, first-instance?)
;; Guest passes index-count=3, other fields none; run returns harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $opt-u32 (option u32))
    (type $opt-s32 (option s32))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $draw-ty (func
      (param "self" $borrow-encoder)
      (param "index-count" u32)
      (param "instance-count" $opt-u32)
      (param "first-index" $opt-u32)
      (param "base-vertex" $opt-s32)
      (param "first-instance" $opt-u32)))
    (export "[method]gpu-render-bundle-encoder.draw-indexed" (func (type $draw-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
  ))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.draw-indexed" (func $draw-ix))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $d_lower (canon lower (func $draw-ix)))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "draw-ix"
      (func $draw-ix (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)))
    (func (export "run") (result i32)
      (call $draw-ix
        (call $get-encoder)
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
      (export "get-encoder" (func $ge_lower))
      (export "draw-ix" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
