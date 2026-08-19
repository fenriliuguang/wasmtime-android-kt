;; S6+: [constructor]record-option-gpu-size64 + [method]record-option-gpu-size64.keys
;; Guest constructs record then keys; harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "record-option-gpu-size64" (type $rec (sub resource)))
    (type $own-rec (own $rec))
    (export "[constructor]record-option-gpu-size64" (func (result $own-rec)))
    (type $borrow-rec (borrow $rec))
    (type $list-str (list string))
    (type $meth-ty (func (param "self" $borrow-rec) (result $list-str)))
    (export "[method]record-option-gpu-size64.keys" (func (type $meth-ty)))
  ))
  (alias export $webgpu "[constructor]record-option-gpu-size64" (func $ctor))
  (alias export $webgpu "[method]record-option-gpu-size64.keys" (func $meth))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $c_lower (canon lower (func $ctor)))
  (core func $m_lower
    (canon lower (func $meth)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "ctor" (func $ctor (result i32)))
    (import "" "meth" (func $meth (param i32 i32)))
    (func (export "run") (result i32)
      (call $meth (call $ctor) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "ctor" (func $c_lower))
      (export "meth" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
