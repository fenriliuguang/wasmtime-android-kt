;; S6+: get-adapter-info + [method]gpu-adapter-info.is-fallback-adapter
;; WIT: is-fallback-adapter: func() -> bool. Host returns false; harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-adapter-info" (type $gpu-adapter-info (sub resource)))
    (type $borrow-info (borrow $gpu-adapter-info))
    (type $ty (func (param "self" $borrow-info) (result bool)))
    (export "[method]gpu-adapter-info.is-fallback-adapter" (func (type $ty)))
    (type $own-info (own $gpu-adapter-info))
    (export "get-adapter-info" (func (result $own-info)))
  ))
  (alias export $webgpu "get-adapter-info" (func $get-info))
  (alias export $webgpu "[method]gpu-adapter-info.is-fallback-adapter" (func $fb))

  (core func $gi_lower (canon lower (func $get-info)))
  (core func $f_lower (canon lower (func $fb)))

  (core module $m
    (import "" "get-info" (func $get-info (result i32)))
    (import "" "fb" (func $fb (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $fb (call $get-info)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-info" (func $gi_lower))
      (export "fb" (func $f_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
