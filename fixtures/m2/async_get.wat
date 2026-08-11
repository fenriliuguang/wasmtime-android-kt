(component
  (import "get" (func $get async (result u32)))

  (core module $m
    (import "" "get" (func $get (result i32)))
    (func (export "run") (result i32)
      call $get
    )
  )
  (core func $get_lower (canon lower (func $get)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get" (func $get_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
