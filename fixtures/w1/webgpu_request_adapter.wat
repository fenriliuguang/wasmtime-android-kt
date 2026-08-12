;; W1: wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter (transitional flat name).
;; Same sync u32 L2 path as experimental; NOT final `[method]gpu.request-adapter`.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "request-adapter" (func (result u32)))
  ))
  (alias export $webgpu "request-adapter" (func $request-adapter))

  (core module $m
    (import "" "request-adapter" (func $request-adapter (result i32)))
    (func (export "run") (result i32)
      call $request-adapter
    )
  )
  (core func $ra_lower (canon lower (func $request-adapter)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "request-adapter" (func $ra_lower))
    ))
  ))
  (func (export "run") (result u32)
    (canon lift (core func $i "run"))
  )
)
