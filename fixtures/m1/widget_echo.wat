(component
  (import "widget" (type $Widget (sub resource)))
  (import "make-widget" (func $make (param "rep" u32) (result (own $Widget))))
  (import "echo-widget" (func $echo (param "w" (borrow $Widget)) (result u32)))

  (core module $m
    (import "" "make" (func $make (param i32) (result i32)))
    (import "" "echo" (func $echo (param i32) (result i32)))
    (func (export "run") (param i32) (result i32)
      (local $h i32)
      local.get 0
      call $make
      local.set $h
      local.get $h
      call $echo
    )
  )
  (core func $make_lower (canon lower (func $make)))
  (core func $echo_lower (canon lower (func $echo)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "make" (func $make_lower))
      (export "echo" (func $echo_lower))
    ))
  ))
  (func (export "run") (param "rep" u32) (result u32)
    (canon lift (core func $i "run"))
  )
)
