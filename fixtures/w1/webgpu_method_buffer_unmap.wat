;; W3+: get-buffer + [method]gpu-buffer.unmap (self; sync void). Returns stub 31.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "get-buffer" (func (result (own $gpu-buffer))))
    (export "[method]gpu-buffer.unmap"
      (func (param "self" (borrow $gpu-buffer))))
  ))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-buffer.unmap" (func $unmap))

  (core module $m
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "unmap" (func $unmap (param i32)))
    (func (export "run") (result i32)
      (local $buf i32)
      (local.set $buf (call $get-buffer))
      (call $unmap (local.get $buf))
      (i32.const 31)
    )
  )
  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $um_lower (canon lower (func $unmap)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-buffer" (func $gb_lower))
      (export "unmap" (func $um_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
