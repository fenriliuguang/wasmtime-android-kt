;; WASI 0.3 package smoke: wasi:clocks/system-clock@0.3.0#now
;; Official instant record {seconds: s64, nanoseconds: u32}.
;; Canon lower stores the record in memory (two scalars > max flat results).
;; Guest `run` returns the seconds field as u64 (unix wall-clock).
;; No timezone in the 0.3.0 pin (system-clock exports now + resolution only).
(component
  (import "wasi:clocks/system-clock@0.3.0" (instance $clock
    (type $instant-def (record (field "seconds" s64) (field "nanoseconds" u32)))
    (export "instant" (type $instant (eq $instant-def)))
    (export "now" (func (result $instant)))
  ))
  (alias export $clock "now" (func $now))

  (core module $libc (memory (export "mem") 1))
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "now" (func $now (param i32)))
    (func (export "run") (result i64)
      (call $now (i32.const 0))
      (i64.load (i32.const 0))
    )
  )
  (core func $now_lower (canon lower (func $now) (memory $libc "mem")))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "now" (func $now_lower))
    ))
  ))
  (func (export "run") (result u64)
    (canon lift (core func $i "run")))
)
