;; S6+: get-buffer + [method]gpu-buffer.size
;; WIT: size: func() -> gpu-size64-out. Host returns 0; harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $size-ty (func (param "self" $borrow-buf) (result u64)))
    (export "[method]gpu-buffer.size" (func (type $size-ty)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "get-buffer" (func $get-buf))
  (alias export $webgpu "[method]gpu-buffer.size" (func $size))

  (core func $gb_lower (canon lower (func $get-buf)))
  (core func $s_lower (canon lower (func $size)))

  (core module $m
    (import "" "get-buf" (func $get-buf (result i32)))
    (import "" "size" (func $size (param i32) (result i64)))
    (func (export "run") (result i32)
      (drop (call $size (call $get-buf)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-buf" (func $gb_lower))
      (export "size" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
