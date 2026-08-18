;; S6+: get-render-bundle-encoder + get-buffer +
;; [method]gpu-render-bundle-encoder.set-vertex-buffer
;; WIT: set-vertex-buffer: func(slot: u32, buffer: option<borrow<gpu-buffer>>,
;;      offset: option<u64>, size: option<u64>)
;; Guest passes slot=0, buffer=some, offset/size=none; drops buffer; run returns 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $opt-buf (option $borrow-buf))
    (type $opt-u64 (option u64))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $set-ty (func
      (param "self" $borrow-encoder)
      (param "slot" u32)
      (param "buffer" $opt-buf)
      (param "offset" $opt-u64)
      (param "size" $opt-u64)))
    (export "[method]gpu-render-bundle-encoder.set-vertex-buffer" (func (type $set-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.set-vertex-buffer" (func $set-vb))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $svb_lower (canon lower (func $set-vb)))
  (core func $db_lower (canon resource.drop $gpu-buffer))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "set-vb"
      (func $set-vb (param i32 i32 i32 i32 i32 i64 i32 i64)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local $buf i32)
      (local.set $encoder (call $get-encoder))
      (local.set $buf (call $get-buffer))
      (call $set-vb
        (local.get $encoder)
        (i32.const 0)
        (i32.const 1) (local.get $buf)
        (i32.const 0) (i64.const 0)
        (i32.const 0) (i64.const 0))
      (call $drop-buffer (local.get $buf))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "get-buffer" (func $gb_lower))
      (export "set-vb" (func $svb_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
