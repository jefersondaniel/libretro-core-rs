# Hardware Rendering and OpenGL

Hardware rendering is negotiated through `Environment`, then used through
`Runtime` and the typed OpenGL wrappers.

During `load_game`, request candidate contexts:

```rust,ignore
let mut env = runtime.environment();
let candidates = opengl_modern_preferred_hw_render_candidates();
if env.set_hw_render_from_candidates(&candidates).is_none() {
    return false;
}
```

The built-in candidate sets are:

- `opengl_modern_preferred_hw_render_candidates()`: generic OpenGL, explicit
  GLES 2.0, legacy GLES2, OpenGL core 3.3, then GLES3.
- `opengl_compatibility_hw_render_candidates()`: generic OpenGL, explicit
  GLES 2.0, then legacy GLES2.

When the frontend creates or recreates a context, `hw_context_reset` should load
symbols and rebuild GL objects. When the context is destroyed, release GL-owned
state in `hw_context_destroy` or `unload_game`.

Per frame, call `runtime.current_framebuffer()` because the frontend-provided
FBO can change. Bind it with `GlFramebuffer::from_raw(framebuffer)`, set the
viewport, render, restore shared GL state, and submit with
`runtime.video_refresh_hw_with_audio(width, height, 0, &audio_frames)`.

If hardware mode is active but a framebuffer or renderer resource is missing,
submit a duplicate hardware frame with audio and surface a diagnostic. Software
pixel fallback is a pre-negotiation path, not a post-negotiation frame path.

The OpenGL API follows recognizable command names with typed arguments:

- `GlBufferTarget`, `GlBufferUsage`, and byte-size newtypes for buffers.
- `GlTextureTarget`, formats, filters, wraps, and dimensions for textures.
- `GlFramebufferTarget`, attachments, renderbuffer formats, and rectangles for
  framebuffer work.
- `GlDrawMode`, `GlDrawRange`, index types, and vertex layouts for drawing.

The examples show two context strategies:

- [Modern OpenGL core](../examples/demo-core.md) prefers modern desktop/GLES
  paths and falls back where needed.
- [Compatibility OpenGL core](../examples/retrocompat-core.md) uses a smaller
  compatibility profile and visible diagnostics.

Tutorial: [OpenGL](../opengl.md).

Reference: [OpenGL Cores](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/opengl-cores.md).
