;; L-FS-TIMES: wasi:filesystem/types@0.3.0 [method]descriptor.set-times / set-times-at
;; Official async func; guest imports sync WIT. Sandbox files only.
;; new-timestamp: no-change | now | timestamp(instant).
;; Guest: open-at file → set-times now/now → no-change/no-change → set-times-at now
;; → set-times-at("..") access. Harness 1.
(component
  (import "wasi:filesystem/types@0.3.0" (instance $types
    (export "descriptor" (type $descriptor (sub resource)))
    (type $error-code-def (variant
      (case "access")
      (case "already")
      (case "bad-descriptor")
      (case "busy")
      (case "deadlock")
      (case "quota")
      (case "exist")
      (case "file-too-large")
      (case "illegal-byte-sequence")
      (case "in-progress")
      (case "interrupted")
      (case "invalid")
      (case "io")
      (case "is-directory")
      (case "loop")
      (case "too-many-links")
      (case "message-size")
      (case "name-too-long")
      (case "no-device")
      (case "no-entry")
      (case "no-lock")
      (case "insufficient-memory")
      (case "insufficient-space")
      (case "not-directory")
      (case "not-empty")
      (case "not-recoverable")
      (case "unsupported")
      (case "no-tty")
      (case "no-such-device")
      (case "overflow")
      (case "not-permitted")
      (case "pipe")
      (case "read-only")
      (case "invalid-seek")
      (case "text-file-busy")
      (case "cross-device")
      (case "other" (option string))
    ))
    (export "error-code" (type $error-code (eq $error-code-def)))
    (type $instant-def (record (field "seconds" s64) (field "nanoseconds" u32)))
    (export "instant" (type $instant (eq $instant-def)))
    (type $new-timestamp-def (variant
      (case "no-change")
      (case "now")
      (case "timestamp" $instant)
    ))
    (export "new-timestamp" (type $new-timestamp (eq $new-timestamp-def)))
    (type $path-flags-def (flags "symlink-follow"))
    (export "path-flags" (type $path-flags (eq $path-flags-def)))
    (type $io-result (result (error $error-code)))
    (type $borrow-desc (borrow $descriptor))
    (type $open-result (result (own $descriptor) (error $error-code)))
    (export "[method]descriptor.set-times"
      (func (param "self" $borrow-desc)
            (param "data-access-timestamp" $new-timestamp)
            (param "data-modification-timestamp" $new-timestamp)
            (result $io-result)))
    (export "[method]descriptor.set-times-at"
      (func (param "self" $borrow-desc)
            (param "path-flags" $path-flags)
            (param "path" string)
            (param "data-access-timestamp" $new-timestamp)
            (param "data-modification-timestamp" $new-timestamp)
            (result $io-result)))
    (export "[method]descriptor.open-at"
      (func (param "self" $borrow-desc) (param "path" string) (result $open-result)))
  ))
  (alias export $types "descriptor" (type $descriptor))
  (alias export $types "error-code" (type $error-code))
  (alias export $types "[method]descriptor.set-times" (func $set-times))
  (alias export $types "[method]descriptor.set-times-at" (func $set-times-at))
  (alias export $types "[method]descriptor.open-at" (func $open-at))
  (import "wasi:filesystem/preopens@0.3.0" (instance $preopens
    (export "descriptor" (type (eq $descriptor)))
    (type $dir-tuple (tuple (own $descriptor) string))
    (export "get-directories" (func (result (list $dir-tuple))))
  ))
  (alias export $preopens "get-directories" (func $get-directories))

  (core module $libc
    (memory (export "mem") 1)
    (data (i32.const 16) "hello.txt")
    (data (i32.const 32) "..")
    (global $last (mut i32) (i32.const 256))
    (func (export "realloc")
      (param $oldptr i32) (param $oldlen i32) (param $align i32) (param $newlen i32)
      (result i32)
      (local $ret i32)
      (local.set $ret (global.get $last))
      (global.set $last
        (i32.and
          (i32.add (i32.add (local.get $ret) (local.get $newlen)) (i32.const 7))
          (i32.const -8)))
      (local.get $ret)
    )
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-directories" (func $get-directories (param i32)))
    (import "" "open-at" (func $open-at (param i32 i32 i32 i32)))
    ;; flattened new-timestamp: disc i32 + instant {s64, u32}
    (import "" "set-times" (func $set-times
      (param i32 i32 i64 i32 i32 i64 i32 i32)))
    (import "" "set-times-at" (func $set-times-at
      (param i32 i32 i32 i32 i32 i64 i32 i32 i64 i32 i32)))

    (func (export "run") (result i32)
      (local $dir i32)
      (local $desc i32)
      (local $list i32)
      (local $len i32)

      (call $get-directories (i32.const 80))
      (local.set $list (i32.load (i32.const 80)))
      (local.set $len (i32.load (i32.const 84)))
      (if (i32.eqz (local.get $len))
        (then (return (i32.const 0))))
      (local.set $dir (i32.load (local.get $list)))

      (call $open-at (local.get $dir) (i32.const 16) (i32.const 9) (i32.const 192))
      (if (i32.ne (i32.load8_u (i32.const 192)) (i32.const 0))
        (then (return (i32.const 0))))
      (local.set $desc (i32.load (i32.const 196)))

      ;; set-times(now, now)
      (call $set-times
        (local.get $desc)
        (i32.const 1) (i64.const 0) (i32.const 0)
        (i32.const 1) (i64.const 0) (i32.const 0)
        (i32.const 256))
      (if (i32.ne (i32.load8_u (i32.const 256)) (i32.const 0))
        (then (return (i32.const 0))))

      ;; set-times(no-change, no-change)
      (call $set-times
        (local.get $desc)
        (i32.const 0) (i64.const 0) (i32.const 0)
        (i32.const 0) (i64.const 0) (i32.const 0)
        (i32.const 256))
      (if (i32.ne (i32.load8_u (i32.const 256)) (i32.const 0))
        (then (return (i32.const 0))))

      ;; set-times-at(0, "hello.txt", now, now)
      (call $set-times-at
        (local.get $dir)
        (i32.const 0)
        (i32.const 16) (i32.const 9)
        (i32.const 1) (i64.const 0) (i32.const 0)
        (i32.const 1) (i64.const 0) (i32.const 0)
        (i32.const 256))
      (if (i32.ne (i32.load8_u (i32.const 256)) (i32.const 0))
        (then (return (i32.const 0))))

      ;; set-times-at("..") → access
      (call $set-times-at
        (local.get $dir)
        (i32.const 0)
        (i32.const 32) (i32.const 2)
        (i32.const 1) (i64.const 0) (i32.const 0)
        (i32.const 1) (i64.const 0) (i32.const 0)
        (i32.const 256))
      (if (i32.ne (i32.load8_u (i32.const 256)) (i32.const 1))
        (then (return (i32.const 0))))
      (if (i32.ne (i32.load8_u (i32.const 260)) (i32.const 0))
        (then (return (i32.const 0))))

      (i32.const 1)
    )
  )

  (core func $get_directories_lower
    (canon lower (func $get-directories)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $open_at_lower
    (canon lower (func $open-at)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $set_times_lower
    (canon lower (func $set-times)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $set_times_at_lower
    (canon lower (func $set-times-at)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "get-directories" (func $get_directories_lower))
      (export "open-at" (func $open_at_lower))
      (export "set-times" (func $set_times_lower))
      (export "set-times-at" (func $set_times_at_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
