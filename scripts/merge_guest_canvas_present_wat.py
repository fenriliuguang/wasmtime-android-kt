#!/usr/bin/env python3
"""Build dawn_guest_canvas_present.wat from dawn_guest_render.wat.

Starts from the already-valid render fixture (named types only) and inserts
gpu-canvas-context configure + get-current-texture. Guest run uses the canvas
swapchain texture instead of create-texture 1x1. Does not merge canvas_context_present.wat
(numeric type indices + duplicate $usage names).
"""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RENDER = ROOT / "fixtures/w1/webgpu_method_dawn_guest_render.wat"
OUT = ROOT / "fixtures/w1/webgpu_method_dawn_guest_canvas_present.wat"

CANVAS_TYPES = """
    (type $canvas-cs (enum "srgb" "display-p3"))
    (export "predefined-color-space" (type $canvas-cs-ex (eq $canvas-cs)))
    (type $canvas-alpha (enum "opaque" "premultiplied"))
    (export "gpu-canvas-alpha-mode" (type $canvas-alpha-ex (eq $canvas-alpha)))
    (type $canvas-tm-mode (enum "standard" "extended"))
    (export "gpu-canvas-tone-mapping-mode" (type $canvas-tm-mode-ex (eq $canvas-tm-mode)))
    (type $opt-canvas-usage (option $tex-usage-ex))
    (type $opt-canvas-cs (option $canvas-cs-ex))
    (type $opt-canvas-tm-mode (option $canvas-tm-mode-ex))
    (type $canvas-tm (record (field "mode" $opt-canvas-tm-mode)))
    (export "gpu-canvas-tone-mapping" (type $canvas-tm-ex (eq $canvas-tm)))
    (type $opt-canvas-tm (option $canvas-tm-ex))
    (type $opt-canvas-alpha (option $canvas-alpha-ex))
    (export "gpu-canvas-context" (type $gpu-canvas-context (sub resource)))
    (type $canvas-cfg (record
      (field "device" $borrow-device)
      (field "format" $gpu-texture-format)
      (field "usage" $opt-canvas-usage)
      (field "view-formats" $opt-list-fmt)
      (field "color-space" $opt-canvas-cs)
      (field "tone-mapping" $opt-canvas-tm)
      (field "alpha-mode" $opt-canvas-alpha)
    ))
    (export "gpu-canvas-configuration" (type $canvas-cfg-ex (eq $canvas-cfg)))
    (type $borrow-ctx (borrow $gpu-canvas-context))
    (type $configure-ty (func
      (param "self" $borrow-ctx)
      (param "configuration" $canvas-cfg-ex)))
    (export "[method]gpu-canvas-context.configure" (func (type $configure-ty)))
    (type $get-tex-ty (func (param "self" $borrow-ctx) (result $own-texture)))
    (export "[method]gpu-canvas-context.get-current-texture" (func (type $get-tex-ty)))
    (type $own-ctx (own $gpu-canvas-context))
    (export "get-canvas-context" (func (result $own-ctx)))
"""

HEADER = """\
;; WG-6: guest-drawn frame via gpu-canvas-context.get-current-texture (not host clear).
;; get-canvas-context + configure + get-device → shader + VERTEX buffer + pipeline →
;; get-current-texture → create-view → render pass → draw(3) → submit → drop owns.
;; Host presents after guest submit. Harness 1. get-* ctors are test-only.
;; Flattened configure is 15 i32s. Options are none.
"""


def main() -> None:
    text = RENDER.read_text(encoding="utf-8")
    # Drop the render-only header comment block (lines starting with ;; until (component).
    body_start = text.index("(component")
    text = HEADER + text[body_start:]

    needle = '    (export "[method]gpu-queue.submit" (func (type $submit-ty)))\n  ))'
    if needle not in text:
        raise SystemExit("submit export close not found")
    text = text.replace(needle, '    (export "[method]gpu-queue.submit" (func (type $submit-ty)))\n' + CANVAS_TYPES + "  ))", 1)

    # Remove create-texture from the guest import surface.
    text = text.replace(
        '    (type $create-tex-ty (func\n'
        '      (param "self" $borrow-device)\n'
        '      (param "descriptor" $tex-desc-ex)\n'
        '      (result $own-texture)))\n'
        '    (export "[method]gpu-device.create-texture" (func (type $create-tex-ty)))\n\n',
        "",
        1,
    )
    text = text.replace(
        '  (alias export $webgpu "[method]gpu-device.create-texture" (func $create-texture))\n',
        '  (alias export $webgpu "gpu-canvas-context" (type $gpu-canvas-context))\n'
        '  (alias export $webgpu "get-canvas-context" (func $get-ctx))\n'
        '  (alias export $webgpu "[method]gpu-canvas-context.configure" (func $configure))\n'
        '  (alias export $webgpu "[method]gpu-canvas-context.get-current-texture" (func $get-tex))\n',
        1,
    )

    text = text.replace(
        '  (core func $gd_lower (canon lower (func $get-device)))\n',
        '  (core func $gd_lower (canon lower (func $get-device)))\n'
        '  (core func $gc_lower (canon lower (func $get-ctx)))\n'
        '  (core func $cf_lower\n'
        '    (canon lower (func $configure)\n'
        '      (memory $builtins "mem")\n'
        '      (realloc (func $builtins "realloc"))))\n'
        '  (core func $gt_lower (canon lower (func $get-tex)))\n',
        1,
    )
    text = text.replace(
        '  (core func $ctex_lower\n'
        '    (canon lower (func $create-texture)\n'
        '      (memory $builtins "mem")\n'
        '      (realloc (func $builtins "realloc"))))\n',
        "",
        1,
    )

    text = text.replace(
        '    (import "" "get-device" (func $get-device (result i32)))\n',
        '    (import "" "get-device" (func $get-device (result i32)))\n'
        '    (import "" "get-canvas-context" (func $get-ctx (result i32)))\n'
        '    (import "" "configure" (func $configure\n'
        '      (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)))\n'
        '    (import "" "get-current-texture" (func $get-tex (param i32) (result i32)))\n',
        1,
    )
    text = text.replace(
        '    (import "" "create-texture" (func $create-texture (param i32) (result i32)))\n',
        "",
        1,
    )

    text = text.replace(
        "      (local $device i32)\n      (local $shader i32)",
        "      (local $device i32)\n      (local $ctx i32)\n      (local $shader i32)",
        1,
    )
    text = text.replace(
        "      (local.set $device (call $get-device))\n      (local.set $shader",
        "      (local.set $device (call $get-device))\n"
        "      (local.set $ctx (call $get-ctx))\n"
        "      (call $configure\n"
        "        (local.get $ctx)\n"
        "        (local.get $device)\n"
        "        (i32.const 21)\n"
        "        (i32.const 0) (i32.const 0)\n"
        "        (i32.const 0) (i32.const 0) (i32.const 0)\n"
        "        (i32.const 0) (i32.const 0)\n"
        "        (i32.const 0) (i32.const 0) (i32.const 0)\n"
        "        (i32.const 0) (i32.const 0))\n"
        "      (local.set $shader",
        1,
    )

    old_tex = """\
      (i32.store (i32.const 0) (local.get $device))
      (i32.store (i32.const 4) (i32.const 1))
      (i32.store (i32.const 8) (i32.const 1))
      (i32.store (i32.const 12) (i32.const 1))
      (i32.store (i32.const 16) (i32.const 1))
      (i32.store (i32.const 20) (i32.const 1))
      (i32.store8 (i32.const 24) (i32.const 1))
      (i32.store (i32.const 28) (i32.const 1))
      (i32.store8 (i32.const 32) (i32.const 1))
      (i32.store (i32.const 36) (i32.const 1))
      (i32.store8 (i32.const 40) (i32.const 1))
      (i32.store8 (i32.const 41) (i32.const 1))
      (i32.store8 (i32.const 42) (i32.const 21))
      (i32.store8 (i32.const 43) (i32.const 16))
      (local.set $texture (call $create-texture (i32.const 0)))"""
    new_tex = "      (local.set $texture (call $get-tex (local.get $ctx)))"
    if old_tex not in text:
        raise SystemExit("create-texture guest block not found")
    text = text.replace(old_tex, new_tex, 1)

    text = text.replace(
        '      (export "get-device" (func $gd_lower))\n',
        '      (export "get-device" (func $gd_lower))\n'
        '      (export "get-canvas-context" (func $gc_lower))\n'
        '      (export "configure" (func $cf_lower))\n'
        '      (export "get-current-texture" (func $gt_lower))\n',
        1,
    )
    text = text.replace(
        '      (export "create-texture" (func $ctex_lower))\n',
        "",
        1,
    )

    OUT.write_text(text, encoding="utf-8")
    print(f"wrote {OUT} ({len(text)} bytes)")


if __name__ == "__main__":
    main()
