;; S6+: get-pass + get-buffer + [method]gpu-render-pass-encoder.draw-indirect
;; WIT: draw-indirect: func(indirect-buffer: borrow, indirect-offset: u64)
;; Guest passes borrow buffer, offset=0; drops buffer; run returns harness 1.
;; L2 still host-fixed draw(3).
;; get-pass / get-buffer are test constructors (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $draw-ty (func
      (param "self" $borrow-pass)
      (param "indirect-buffer" $borrow-buf)
      (param "indirect-offset" u64)))
    (export "[method]gpu-render-pass-encoder.draw-indirect" (func (type $draw-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-render-pass-encoder.draw-indirect" (func $draw-ind))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $d_lower (canon lower (func $draw-ind)))
  (core func $db_lower (canon resource.drop $gpu-buffer))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "draw-ind" (func $draw-ind (param i32 i32 i64)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local $buf i32)
      (local.set $pass (call $get-pass))
      (local.set $buf (call $get-buffer))
      (call $draw-ind (local.get $pass) (local.get $buf) (i64.const 0))
      (call $drop-buffer (local.get $buf))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "get-buffer" (func $gb_lower))
      (export "draw-ind" (func $d_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
