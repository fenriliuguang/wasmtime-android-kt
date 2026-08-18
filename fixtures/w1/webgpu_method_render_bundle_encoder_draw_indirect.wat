;; S6+: get-render-bundle-encoder + get-buffer +
;; [method]gpu-render-bundle-encoder.draw-indirect
;; WIT: draw-indirect: func(indirect-buffer: borrow, indirect-offset: u64)
;; Guest passes borrow buffer, offset=0; drops buffer; run returns harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $draw-ty (func
      (param "self" $borrow-encoder)
      (param "indirect-buffer" $borrow-buf)
      (param "indirect-offset" u64)))
    (export "[method]gpu-render-bundle-encoder.draw-indirect" (func (type $draw-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.draw-indirect" (func $draw-ind))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $d_lower (canon lower (func $draw-ind)))
  (core func $db_lower (canon resource.drop $gpu-buffer))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "draw-ind" (func $draw-ind (param i32 i32 i64)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local $buf i32)
      (local.set $encoder (call $get-encoder))
      (local.set $buf (call $get-buffer))
      (call $draw-ind (local.get $encoder) (local.get $buf) (i64.const 0))
      (call $drop-buffer (local.get $buf))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "get-buffer" (func $gb_lower))
      (export "draw-ind" (func $d_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
