;; WASI 0.3 package smoke: wasi:clocks/monotonic-clock@0.3.0#resolution
;; Host returns duration in nanoseconds (u64); guest exports run.
(component
  (import "wasi:clocks/monotonic-clock@0.3.0" (instance $clock
    (export "resolution" (func (result u64)))
  ))
  (alias export $clock "resolution" (func $resolution))

  (core module $m
    (import "" "resolution" (func $resolution (result i64)))
    (func (export "run") (result i64)
      call $resolution
    )
  )
  (core func $resolution_lower (canon lower (func $resolution)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "resolution" (func $resolution_lower))
    ))
  ))
  (func (export "run") (result u64)
    (canon lift (core func $i "run")))
)
