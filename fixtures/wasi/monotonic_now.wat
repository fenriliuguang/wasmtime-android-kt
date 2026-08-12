;; WASI 0.3 package smoke: wasi:clocks/monotonic-clock@0.3.0#now
;; Host returns a non-decreasing mark (u64 nanoseconds-like); guest exports run.
(component
  (import "wasi:clocks/monotonic-clock@0.3.0" (instance $clock
    (export "now" (func (result u64)))
  ))
  (alias export $clock "now" (func $now))

  (core module $m
    (import "" "now" (func $now (result i64)))
    (func (export "run") (result i64)
      call $now
    )
  )
  (core func $now_lower (canon lower (func $now)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "now" (func $now_lower))
    ))
  ))
  (func (export "run") (result u64)
    (canon lift (core func $i "run")))
)
