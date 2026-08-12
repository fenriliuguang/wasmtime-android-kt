;; WASI 0.3 package smoke: wasi:random/random@0.3.0#get-random-u64
;; Host registers CSPRNG; guest returns one u64 (any value is success).
(component
  (import "wasi:random/random@0.3.0" (instance $random
    (export "get-random-u64" (func (result u64)))
  ))
  (alias export $random "get-random-u64" (func $get-random-u64))

  (core module $m
    (import "" "get-random-u64" (func $get-random-u64 (result i64)))
    (func (export "run") (result i64)
      call $get-random-u64
    )
  )
  (core func $get_lower (canon lower (func $get-random-u64)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-random-u64" (func $get_lower))
    ))
  ))
  (func (export "run") (result u64)
    (canon lift (core func $i "run")))
)
