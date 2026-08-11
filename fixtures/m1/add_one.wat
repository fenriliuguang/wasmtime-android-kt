(component
  (core module $m
    (func (export "run") (param i32) (result i32)
      local.get 0
      i32.const 1
      i32.add
    )
  )
  (core instance $i (instantiate $m))
  (func (export "run") (param "a" u32) (result u32)
    (canon lift (core func $i "run"))
  )
)
