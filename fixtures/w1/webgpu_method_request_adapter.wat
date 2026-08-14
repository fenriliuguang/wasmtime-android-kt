;; W3: wasi:webgpu/webgpu@0.3.0-rc.2 get-gpu + [method]gpu.request-adapter
;; (resource self; true CM async). Transitional: method returns u32 (not
;; option<gpu-adapter>); no options record. Flat request-adapter stays registered.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu" (type $gpu (sub resource)))
    (export "get-gpu" (func (result (own $gpu))))
    (export "[method]gpu.request-adapter"
      (func async (param "self" (borrow $gpu)) (result u32)))
  ))
  (alias export $webgpu "get-gpu" (func $get-gpu))
  (alias export $webgpu "[method]gpu.request-adapter" (func $request-adapter))

  (core module $m
    (import "" "get-gpu" (func $get-gpu (result i32)))
    (import "" "request-adapter" (func $request-adapter (param i32) (result i32)))
    (func (export "run") (result i32)
      (local $gpu i32)
      (local.set $gpu (call $get-gpu))
      (call $request-adapter (local.get $gpu))
    )
  )
  (core func $gg_lower (canon lower (func $get-gpu)))
  (core func $ra_lower (canon lower (func $request-adapter)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-gpu" (func $gg_lower))
      (export "request-adapter" (func $ra_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
