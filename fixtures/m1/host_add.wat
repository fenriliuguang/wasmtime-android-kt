(component
  (import "add" (func $add (param "a" u32) (param "b" u32) (result u32)))
  (core module $m
    (import "" "add" (func $add (param i32 i32) (result i32)))
    (func (export "run") (param i32 i32) (result i32)
      local.get 0
      local.get 1
      call $add
    )
  )
  (core func $add_lower (canon lower (func $add)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "add" (func $add_lower))
    ))
  ))
  (func (export "run") (param "a" u32) (param "b" u32) (result u32)
    (canon lift (core func $i "run"))
  )
)
