# Hardware Rendering and OpenGL

Hardware rendering is negotiated through `Environment`, then used through
`Runtime` and the typed `Gl` wrapper. The frontend owns the OpenGL context and
the current framebuffer. The core owns its GL objects and must recreate them
when the frontend recreates the context.

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
symbols and rebuild GL objects:

```rust,ignore
fn hw_context_reset(&mut self, runtime: &mut Runtime<'_>) {
    let gl = Gl::init(runtime)
        .unwrap_or_else(|error| panic!("failed to initialize GL: {error}"));
    self.gl = Some(gl);
}
```

`Gl::init` has staged behavior:

- Mandatory symbols: framebuffer binding, viewport, clear color, and clear.
- Optional symbols: shaders, buffers, textures, blending, vertex arrays, and
  richer GL helpers.
- Resource and draw methods return `Err` when their feature is unavailable.
- State restoration and cleanup methods no-op when their feature is unavailable.

That means a core can always try to present a clear-only diagnostic frame before
probing richer rendering.

Per frame, call `runtime.current_framebuffer()` because the frontend-provided
FBO can change. Bind it with `GlFramebuffer::from_raw(framebuffer)`, set the
viewport, render, restore shared GL state, and submit with
`runtime.video_refresh_hw_with_audio(width, height, 0, &audio_frames)`.

Minimal frame:

```rust,ignore
let Some(framebuffer) = runtime.current_framebuffer() else {
    let _ = runtime.video_refresh_dupe_with_audio(width, height, audio);
    return;
};

gl.bind_framebuffer(
    GlFramebufferTarget::Framebuffer,
    GlFramebuffer::from_raw(framebuffer),
)?;
gl.viewport(GlRect::new(0, 0, width, height))?;
gl.clear_color(0.08, 0.09, 0.12, 1.0);
gl.clear_color_buffer();
gl.unbind_framebuffer(GlFramebufferTarget::Framebuffer);
let _ = runtime.video_refresh_hw_with_audio(width, height, 0, audio);
```

Typical buffer upload:

```rust,ignore
let vbo = gl.gen_buffer()?;
gl.bind_buffer(GlBufferTarget::ArrayBuffer, Some(vbo));
gl.buffer_data(
    GlBufferTarget::ArrayBuffer,
    vertices,
    GlBufferUsage::StaticDraw,
)?;
gl.unbind_buffer(GlBufferTarget::ArrayBuffer);
```

Typical draw:

```rust,ignore
gl.use_program(Some(program));
gl.bind_buffer(GlBufferTarget::ArrayBuffer, Some(vbo));
gl.enable_vertex_attrib(position);
gl.vertex_attrib_pointer_f32(position, position_layout);
gl.draw_arrays(GlDrawMode::Triangles, GlDrawRange::from_start(vertex_count))?;
gl.disable_vertex_attrib(position);
gl.unbind_buffer(GlBufferTarget::ArrayBuffer);
gl.use_no_program();
```

Typical texture setup:

```rust,ignore
let texture = gl.gen_texture()?;
gl.active_texture(GlTextureUnit::ZERO)?;
gl.bind_texture(GlTextureTarget::Texture2D, Some(texture));
gl.tex_min_filter(GlTextureTarget::Texture2D, GlTextureMinFilter::Nearest);
gl.tex_mag_filter(GlTextureTarget::Texture2D, GlTextureMagFilter::Nearest);
gl.tex_image_2d(
    GlTextureTarget::Texture2D,
    GlTextureInternalFormat::Rgba,
    GlTextureLevel::ZERO,
    GlTextureSize2D::new(width, height),
    GlTextureFormat::Rgba,
    GlTextureDataType::UnsignedByte,
    Some(rgba_bytes),
)?;
gl.unbind_texture(GlTextureTarget::Texture2D);
```

If hardware mode is active but a framebuffer or renderer resource is missing,
submit a duplicate hardware frame with audio and surface a diagnostic. Software
pixel fallback is a pre-negotiation path, not a post-negotiation frame path.

The OpenGL API follows recognizable command names with typed arguments:

- `GlBufferTarget`, `GlBufferUsage`, and byte-size newtypes for buffers.
- `GlTextureTarget`, formats, filters, wraps, and dimensions for textures.
- `GlFramebufferTarget`, attachments, renderbuffer formats, and rectangles for
  framebuffer work.
- `GlDrawMode`, `GlDrawRange`, index types, and vertex layouts for drawing.

Common feature decisions:

```rust,ignore
if gl.supports_shader_pipeline() {
    // Build or draw shader/buffer geometry.
}
if gl.supports_textures() {
    // Upload textures or draw bitmap text.
}
if gl.supports_vertex_arrays() {
    // Use VAOs for core-profile desktop GL.
}
```

The examples show two context strategies:

- [Modern OpenGL core](../examples/demo-core.md) prefers modern desktop/GLES
  paths and falls back where needed.
- [Compatibility OpenGL core](../examples/retrocompat-core.md) uses a smaller
  compatibility profile and visible diagnostics.

Tutorial: [OpenGL](../opengl.md).

Reference: [OpenGL Cores](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/opengl-cores.md).
