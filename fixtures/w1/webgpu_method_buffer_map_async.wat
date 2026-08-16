;; W3+: get-buffer + [method]gpu-buffer.map-async (self; true CM async void).
;; Returns stub 31. Host-fixed MAP_READ; not proposal result/error.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "get-buffer" (func (result (own $gpu-buffer))))
    (export "[method]gpu-buffer.map-async"
      (func async (param "self" (borrow $gpu-buffer))))
  ))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-buffer.map-async" (func $map-async))

  (core module $m
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "map-async" (func $map-async (param i32)))
    (func (export "run") (result i32)
      (local $buf i32)
      (local.set $buf (call $get-buffer))
      (call $map-async (local.get $buf))
      (i32.const 31)
    )
  )
  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $ma_lower (canon lower (func $map-async)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-buffer" (func $gb_lower))
      (export "map-async" (func $ma_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
