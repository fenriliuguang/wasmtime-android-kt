;; S6+: get-device + [method]gpu-device.pop-error-scope
;; WIT: async pop-error-scope: func() -> result<option<gpu-error>, pop-error-scope-error>.
;; Host returns ok/none; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (export "gpu-error" (type $gpu-error (sub resource)))
    (type $own-err (own $gpu-error))
    (type $opt-err (option $own-err))
    (type $kind (variant (case "operation-error")))
    (export "pop-error-scope-error-kind" (type $pop-error-scope-error-kind (eq $kind)))
    (type $pop-err (record (field "kind" $pop-error-scope-error-kind) (field "message" string)))
    (export "pop-error-scope-error" (type $pop-error-scope-error (eq $pop-err)))
    (type $result-opt (result $opt-err (error $pop-error-scope-error)))
    (type $borrow-dev (borrow $gpu-device))
    (type $pop-ty (func async (param "self" $borrow-dev) (result $result-opt)))
    (export "[method]gpu-device.pop-error-scope" (func (type $pop-ty)))
    (type $own-dev (own $gpu-device))
    (export "get-device" (func (result $own-dev)))
  ))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.pop-error-scope" (func $pop))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $pop_lower
    (canon lower (func $pop)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "pop" (func $pop (param i32 i32)))
    (func (export "run") (result i32)
      (call $pop (call $get-device) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-device" (func $gd_lower))
      (export "pop" (func $pop_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
