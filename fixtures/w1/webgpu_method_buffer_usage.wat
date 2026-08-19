;; S6+: get-buffer + [method]gpu-buffer.usage
;; WIT: usage: func() -> gpu-buffer-usage. Host usage bits; harness 1.
;; L2 described buffer handle → usage (stub MAP_READ|COPY_DST when get-buffer rep=0).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $usage (flags "map-read" "map-write" "copy-src" "copy-dst" "index" "vertex" "uniform" "storage" "indirect" "query-resolve"))
    (export "gpu-buffer-usage" (type $gpu-buffer-usage (eq $usage)))
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $usage-fn (func (param "self" $borrow-buf) (result $gpu-buffer-usage)))
    (export "[method]gpu-buffer.usage" (func (type $usage-fn)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "get-buffer" (func $get-buf))
  (alias export $webgpu "[method]gpu-buffer.usage" (func $usage))

  (core func $gb_lower (canon lower (func $get-buf)))
  (core func $u_lower (canon lower (func $usage)))

  (core module $m
    (import "" "get-buf" (func $get-buf (result i32)))
    (import "" "usage" (func $usage (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $usage (call $get-buf)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-buf" (func $gb_lower))
      (export "usage" (func $u_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
