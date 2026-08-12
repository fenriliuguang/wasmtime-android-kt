;; WASI 0.3 package smoke: wasi:clocks/monotonic-clock@0.3.0#wait-for
;; Guest awaits host async wait-for (~2ms), then returns constant 1.
(component
  (import "wasi:clocks/monotonic-clock@0.3.0" (instance $clock
    (export "wait-for" (func async (param "how-long" u64)))
  ))
  (alias export $clock "wait-for" (func $wait-for))

  (core module $m
    (import "" "wait-for" (func $wait-for (param i64)))
    (func (export "run") (result i32)
      ;; 2_000_000 ns = 2ms
      i64.const 2000000
      call $wait-for
      i32.const 1
    )
  )
  (core func $wait_for_lower (canon lower (func $wait-for)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "wait-for" (func $wait_for_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
