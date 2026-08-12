;; WASI 0.3 package smoke: wasi:clocks/monotonic-clock@0.3.0#wait-until
;; Guest: now → add ~2ms → wait-until that instant → return 1.
(component
  (import "wasi:clocks/monotonic-clock@0.3.0" (instance $clock
    (export "now" (func (result u64)))
    (export "wait-until" (func async (param "when" u64)))
  ))
  (alias export $clock "now" (func $now))
  (alias export $clock "wait-until" (func $wait-until))

  (core module $m
    (import "" "now" (func $now (result i64)))
    (import "" "wait-until" (func $wait-until (param i64)))
    (func (export "run") (result i32)
      call $now
      i64.const 2000000
      i64.add
      call $wait-until
      i32.const 1
    )
  )
  (core func $now_lower (canon lower (func $now)))
  (core func $wait_until_lower (canon lower (func $wait-until)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "now" (func $now_lower))
      (export "wait-until" (func $wait_until_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
