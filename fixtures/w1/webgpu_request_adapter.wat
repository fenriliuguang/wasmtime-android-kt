;; W2: wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter (transitional flat name).
;; True CM async import + async export; u32 L2 path (not full option/resource / [method]gpu.*).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "request-adapter" (func async (result u32)))
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
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
