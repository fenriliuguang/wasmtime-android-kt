;; S6+: get-buffer + [method]gpu-buffer.map-state
;; WIT: map-state: func() -> gpu-buffer-map-state. Host returns unmapped; harness 1.
;; L2 described buffer handle → map-state (stub unmapped when get-buffer rep=0).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $ms (enum "unmapped" "pending" "mapped"))
    (export "gpu-buffer-map-state" (type $gpu-buffer-map-state (eq $ms)))
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $ms-fn (func (param "self" $borrow-buf) (result $gpu-buffer-map-state)))
    (export "[method]gpu-buffer.map-state" (func (type $ms-fn)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "get-buffer" (func $get-buf))
  (alias export $webgpu "[method]gpu-buffer.map-state" (func $map-state))

  (core func $gb_lower (canon lower (func $get-buf)))
  (core func $m_lower (canon lower (func $map-state)))

  (core module $m
    (import "" "get-buf" (func $get-buf (result i32)))
    (import "" "map-state" (func $map-state (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $map-state (call $get-buf)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-buf" (func $gb_lower))
      (export "map-state" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
