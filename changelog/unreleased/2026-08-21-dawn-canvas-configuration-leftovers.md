### Code — L2 gpu-canvas-context configure leftovers guest fields to host (2026-08-21)

- JNI `canvasContextConfigureDescribed` is now `(IIII[IIII)I`: view-formats list plus color-space / tone-mapping / alpha-mode (`-1` = absent) on top of device+format+usage
- Fixture `webgpu_method_canvas_context_configure` lifts view-formats=[rgba8unorm], color-space=srgb, tone-mapping=standard, alpha-mode=premultiplied; do not re-cut canvas first-cut or present cite
- androidx `1.0.0-alpha05` `GPUSurfaceConfiguration` takes viewFormats / alphaMode; color-space and tone-mapping have no surface-config slot (stored on the Kotlin leftover record)
