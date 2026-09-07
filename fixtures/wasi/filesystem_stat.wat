;; L-FS-STAT: wasi:filesystem/types@0.3.0 [method]descriptor.stat / stat-at
;; Official packages: types@0.3.0 + preopens@0.3.0. Sandbox descriptor only.
;; Guest: get-directories → stat dir → open-at("hello.txt") → stat file →
;; stat-at("hello.txt") → stat-at("..") access. Harness 1.
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
    (type $descriptor-type-def (variant
      (case "block-device")
      (case "character-device")
      (case "directory")
      (case "fifo")
      (case "symbolic-link")
      (case "regular-file")
      (case "socket")
      (case "other" (option string))
    ))
    (export "descriptor-type" (type $descriptor-type (eq $descriptor-type-def)))
    (type $descriptor-stat-def (record
      (field "type" $descriptor-type)
      (field "link-count" u64)
      (field "size" u64)
      (field "data-access-timestamp" (option $instant))
      (field "data-modification-timestamp" (option $instant))
      (field "status-change-timestamp" (option $instant))
    ))
    (export "descriptor-stat" (type $descriptor-stat (eq $descriptor-stat-def)))
    (type $path-flags-def (flags "symlink-follow"))
    (export "path-flags" (type $path-flags (eq $path-flags-def)))
    (type $stat-result (result $descriptor-stat (error $error-code)))
    (type $borrow-desc (borrow $descriptor))
    (type $open-result (result (own $descriptor) (error $error-code)))
    (export "[method]descriptor.stat"
      (func (param "self" $borrow-desc) (result $stat-result)))
    (export "[method]descriptor.stat-at"
      (func (param "self" $borrow-desc) (param "path-flags" $path-flags) (param "path" string) (result $stat-result)))
    (export "[method]descriptor.open-at"
      (func (param "self" $borrow-desc) (param "path" string) (result $open-result)))
  ))
  (alias export $types "descriptor" (type $descriptor))
  (alias export $types "error-code" (type $error-code))
  (alias export $types "[method]descriptor.stat" (func $stat))
  (alias export $types "[method]descriptor.stat-at" (func $stat-at))
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
    (global $last (mut i32) (i32.const 512))
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
    (import "" "stat" (func $stat (param i32 i32)))
    (import "" "stat-at" (func $stat-at (param i32 i32 i32 i32 i32)))

    (func (export "run") (result i32)
      (local $dir i32)
      (local $desc i32)
      (local $list i32)
      (local $len i32)

      ;; list<tuple<own, string>> at mem[80]: ptr, len.
      (call $get-directories (i32.const 80))
      (local.set $list (i32.load (i32.const 80)))
      (local.set $len (i32.load (i32.const 84)))
      (if (i32.eqz (local.get $len))
        (then (return (i32.const 0))))
      (local.set $dir (i32.load (local.get $list)))

      ;; result<descriptor-stat, error-code> at 256: disc at +0 (align 8),
      ;; type disc at +8 (directory = 2).
      (call $stat (local.get $dir) (i32.const 256))
      (if (i32.ne (i32.load8_u (i32.const 256)) (i32.const 0))
        (then (return (i32.const 0))))
      (if (i32.ne (i32.load8_u (i32.const 264)) (i32.const 2))
        (then (return (i32.const 0))))

      ;; open-at("hello.txt") → empty file. result disc at 192, handle at 196.
      (call $open-at (local.get $dir) (i32.const 16) (i32.const 9) (i32.const 192))
      (if (i32.ne (i32.load8_u (i32.const 192)) (i32.const 0))
        (then (return (i32.const 0))))
      (local.set $desc (i32.load (i32.const 196)))

      ;; file stat: regular-file = 5; size u64 at payload+24 = 256+8+24 = 288.
      (call $stat (local.get $desc) (i32.const 256))
      (if (i32.ne (i32.load8_u (i32.const 256)) (i32.const 0))
        (then (return (i32.const 0))))
      (if (i32.ne (i32.load8_u (i32.const 264)) (i32.const 5))
        (then (return (i32.const 0))))
      (if (i64.ne (i64.load (i32.const 288)) (i64.const 0))
        (then (return (i32.const 0))))

      ;; stat-at(path-flags=0, "hello.txt") same file.
      (call $stat-at (local.get $dir) (i32.const 0) (i32.const 16) (i32.const 9) (i32.const 256))
      (if (i32.ne (i32.load8_u (i32.const 256)) (i32.const 0))
        (then (return (i32.const 0))))
      (if (i32.ne (i32.load8_u (i32.const 264)) (i32.const 5))
        (then (return (i32.const 0))))

      ;; stat-at("..") → error-code.access (result disc 1, variant disc 0 at +8).
      (call $stat-at (local.get $dir) (i32.const 0) (i32.const 32) (i32.const 2) (i32.const 256))
      (if (i32.ne (i32.load8_u (i32.const 256)) (i32.const 1))
        (then (return (i32.const 0))))
      (if (i32.ne (i32.load8_u (i32.const 264)) (i32.const 0))
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
  (core func $stat_lower
    (canon lower (func $stat)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $stat_at_lower
    (canon lower (func $stat-at)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "get-directories" (func $get_directories_lower))
      (export "open-at" (func $open_at_lower))
      (export "stat" (func $stat_lower))
      (export "stat-at" (func $stat_at_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
