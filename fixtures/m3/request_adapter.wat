(component
  (import "experimental:webgpu-cm/host@0.8.0" (instance $host
    (export "request-adapter" (func (result u32)))
  ))
  (alias export $host "request-adapter" (func $request-adapter))

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
