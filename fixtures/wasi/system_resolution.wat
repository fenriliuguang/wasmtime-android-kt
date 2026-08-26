;; WASI 0.3 package smoke: wasi:clocks/system-clock@0.3.0#resolution
;; Official instant record {seconds: s64, nanoseconds: u32}.
;; Canon lower stores the record in memory (two scalars > max flat results).
;; Host resolution is 1 ns → {seconds: 0, nanoseconds: 1}; guest `run` returns ns.
;; No timezone in the 0.3.0 pin.
(component
  (import "wasi:clocks/system-clock@0.3.0" (instance $clock
    (type $instant-def (record (field "seconds" s64) (field "nanoseconds" u32)))
    (export "instant" (type $instant (eq $instant-def)))
    (export "resolution" (func (result $instant)))
  ))
  (alias export $clock "resolution" (func $resolution))

  (core module $libc (memory (export "mem") 1))
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "resolution" (func $resolution (param i32)))
    (func (export "run") (result i64)
      (call $resolution (i32.const 0))
      (if (i64.ne (i64.load (i32.const 0)) (i64.const 0))
        (then unreachable))
      (i64.extend_i32_u (i32.load (i32.const 8)))
    )
  )
  (core func $resolution_lower (canon lower (func $resolution) (memory $libc "mem")))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "resolution" (func $resolution_lower))
    ))
  ))
  (func (export "run") (result u64)
    (canon lift (core func $i "run")))
)
