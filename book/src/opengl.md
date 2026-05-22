# OpenGL

Libretro hardware rendering is a frontend-owned OpenGL or OpenGL ES context.
The core negotiates a context, initializes GL state when the frontend creates
the context, renders into the frontend-provided framebuffer each frame, and
deletes GL-owned objects before the context is gone.

The important difference from SDL, GLFW, or a standalone OpenGL app is that the
core does not create the window or the GL context. The frontend does. The core
asks for a context, receives callbacks when it is available, and looks up GL
symbols through libretro.

`libretro-core` exposes that as one typed handle: `Gl`.

- `Gl::init(runtime)` is called after the context reset callback.
- The clear/framebuffer path is mandatory so a core can always try to show a
  legal hardware frame.
- Shader, buffer, texture, blending, vertex-array, and other richer symbols are
  optional. Methods that create resources or draw return errors when their GL
  feature is unavailable. State cleanup methods no-op when the feature is not
  loaded.

This lets a core start with a visible clear-only diagnostic, then layer in a
triangle, textures, text, or a full renderer as symbols are available.

## Core State

Store GL-owned handles on your core and clear them when the context is
destroyed. Handles are only valid for the current frontend-owned context.

```rust,ignore
struct MyCore {
    gl: Option<Gl>,
    program: Option<GlProgram>,
    vbo: Option<GlBuffer>,
}
```

## Negotiate

Request hardware rendering from `load_game`:

```rust,ignore
fn load_game(&mut self, _game: Option<GameInfo<'_>>, runtime: &mut Runtime<'_>) -> bool {
    let mut env = runtime.environment();
    if !env.set_pixel_format(PixelFormat::Xrgb8888) {
        return false;
    }

    let candidates = opengl_modern_preferred_hw_render_candidates();
    env.set_hw_render_from_candidates(&candidates).is_some()
}
```

Use `opengl_modern_preferred_hw_render_candidates()` for the modern demo path.
Use `opengl_compatibility_hw_render_candidates()` when targeting the smaller
compatibility/diagnostic path.

If negotiation fails, software video is still allowed because hardware mode was
not accepted yet. After hardware mode is accepted, submit hardware or duplicate
frames only.

## Reset

Load symbols and rebuild context-owned objects in `hw_context_reset`:

```rust,ignore
fn hw_context_reset(&mut self, runtime: &mut Runtime<'_>) {
    let gl = Gl::init(runtime)
        .unwrap_or_else(|error| panic!("failed to load OpenGL symbols: {error}"));

    let program = gl
        .build_program(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE)
        .unwrap_or_else(|error| panic!("failed to build GL program: {error}"));

    self.gl = Some(gl);
    self.program = Some(program);
}
```

`Gl::init` loads the legal clear/framebuffer path first and treats richer
shader, buffer, texture, and blend symbols as optional. Methods that only
restore state or clean up unavailable features are no-ops; methods that need
missing rendering resources return a diagnosable error so cores can fall back.

The frontend may recreate the context. Treat the symbol table, programs,
buffers, textures, vertex arrays, and framebuffer-dependent state as
context-lifetime state.

## Clear First

A clear-only frame is the smallest useful hardware-rendered output. It is also
the best first diagnostic because it proves that negotiation, framebuffer
lookup, binding, viewport setup, and frame submission all work.

```rust,ignore
fn draw_clear_frame(&self, runtime: &mut Runtime<'_>, gl: &Gl, audio: &[[i16; 2]]) {
    let Some(framebuffer) = runtime.current_framebuffer() else {
        let _ = runtime.video_refresh_dupe_with_audio(WIDTH, HEIGHT, audio);
        return;
    };

    if gl
        .bind_framebuffer(
            GlFramebufferTarget::Framebuffer,
            GlFramebuffer::from_raw(framebuffer),
        )
        .is_err()
    {
        let _ = runtime.video_refresh_dupe_with_audio(WIDTH, HEIGHT, audio);
        return;
    }

    let _ = gl.viewport(GlRect::new(0, 0, WIDTH, HEIGHT));
    gl.clear_color(0.08, 0.09, 0.12, 1.0);
    gl.clear_color_buffer();
    gl.unbind_framebuffer(GlFramebufferTarget::Framebuffer);

    let _ = runtime.video_refresh_hw_with_audio(WIDTH, HEIGHT, 0, audio);
}
```

Set the viewport every frame. The frontend may reuse the context for its own
work, and `current_framebuffer()` can return a different FBO on different
frames.

## Build A Program And Buffer

Once the clear path works, build renderer resources in `hw_context_reset`.
Program creation, buffer creation, and buffer upload can fail if the frontend
does not expose the required symbols. Keep the error visible instead of silently
continuing to a black frame.

```rust,ignore
fn hw_context_reset(&mut self, runtime: &mut Runtime<'_>) {
    self.destroy_gl_state();

    let gl = Gl::init(runtime)
        .unwrap_or_else(|error| panic!("failed to load OpenGL symbols: {error}"));

    let (vertex_source, fragment_source) = shader_sources_for(gl.context_type());
    let program = gl
        .build_program(vertex_source, fragment_source)
        .unwrap_or_else(|error| panic!("failed to build GL program: {error}"));

    let vertices: [f32; 15] = [
         0.0,  0.6, 1.0, 0.0, 0.0,
        -0.6, -0.6, 0.0, 1.0, 0.0,
         0.6, -0.6, 0.0, 0.0, 1.0,
    ];

    let vbo = gl
        .gen_buffer()
        .unwrap_or_else(|error| panic!("failed to create GL buffer: {error}"));
    gl.bind_buffer(GlBufferTarget::ArrayBuffer, Some(vbo));
    gl.buffer_data(
        GlBufferTarget::ArrayBuffer,
        &vertices,
        GlBufferUsage::StaticDraw,
    )
    .unwrap_or_else(|error| panic!("failed to upload GL buffer: {error}"));
    gl.unbind_buffer(GlBufferTarget::ArrayBuffer);

    self.program = Some(program);
    self.vbo = Some(vbo);
    self.gl = Some(gl);
}
```

Use shader sources that match the negotiated context family. For example, a
core-profile desktop context needs modern `in`/`out` syntax, while GLES2 and
desktop compatibility contexts can use attribute/varying style shaders.

## Render

Ask for the current framebuffer every frame. It can change between frames.

```rust,ignore
let Some(framebuffer) = runtime.current_framebuffer() else {
    let _ = runtime.video_refresh_dupe_with_audio(WIDTH, HEIGHT, &self.audio);
    return;
};

let Some(gl) = self.gl.as_ref() else {
    let _ = runtime.video_refresh_dupe_with_audio(WIDTH, HEIGHT, &self.audio);
    return;
};

if gl
    .bind_framebuffer(
        GlFramebufferTarget::Framebuffer,
        GlFramebuffer::from_raw(framebuffer),
    )
    .is_err()
{
    let _ = runtime.video_refresh_dupe_with_audio(WIDTH, HEIGHT, &self.audio);
    return;
}

let _ = gl.viewport(GlRect::new(0, 0, WIDTH, HEIGHT));
gl.clear_color(0.08, 0.09, 0.12, 1.0);
gl.clear_color_buffer();
gl.unbind_framebuffer(GlFramebufferTarget::Framebuffer);

let _ = runtime.video_refresh_hw_with_audio(WIDTH, HEIGHT, 0, &self.audio);
```

For a triangle draw, bind the program and buffer, configure typed vertex
attributes, draw, then restore the shared state you touched:

```rust,ignore
let Some(program) = self.program else {
    return Ok(());
};
let Some(vbo) = self.vbo else {
    return Ok(());
};

gl.use_program(Some(program));
gl.bind_buffer(GlBufferTarget::ArrayBuffer, Some(vbo));
let position = gl.required_attrib_location(program, "a_pos")?;
let color = gl.required_attrib_location(program, "a_color")?;

gl.enable_vertex_attrib(position);
gl.vertex_attrib_pointer_f32(
    position,
    GlVertexAttribF32Layout::interleaved(GlVertexAttribF32Components::Two, 5)?,
);
gl.enable_vertex_attrib(color);
gl.vertex_attrib_pointer_f32(
    color,
    GlVertexAttribF32Layout::interleaved(GlVertexAttribF32Components::Three, 5)?
        .with_offset_components(GlVertexAttribF32Components::Two),
);

gl.draw_arrays(GlDrawMode::Triangles, GlDrawRange::from_start(3))?;

gl.disable_vertex_attrib(position);
gl.disable_vertex_attrib(color);
gl.unbind_buffer(GlBufferTarget::ArrayBuffer);
gl.use_no_program();
```

Do not leave shared GL state such as framebuffers, buffers, textures, vertex
arrays, enabled attributes, or programs bound when handing the frame back to the
frontend.

After hardware rendering is active, fallback frames should be hardware or
duplicate submissions with audio. Software-pixel fallback is only appropriate
before hardware negotiation succeeds.

## Upload A Texture

Texture helpers use typed targets, formats, filters, wraps, levels, and sizes.
If texture symbols are not available, resource creation and upload methods
return an error and cleanup calls no-op.

```rust,ignore
let texture = gl.gen_texture()?;
gl.active_texture(GlTextureUnit::ZERO)?;
gl.bind_texture(GlTextureTarget::Texture2D, Some(texture));
gl.pixel_store_unpack_alignment(GlPixelStoreAlignment::One);
gl.tex_min_filter(GlTextureTarget::Texture2D, GlTextureMinFilter::Nearest);
gl.tex_mag_filter(GlTextureTarget::Texture2D, GlTextureMagFilter::Nearest);
gl.tex_wrap_s(GlTextureTarget::Texture2D, GlTextureWrap::ClampToEdge);
gl.tex_wrap_t(GlTextureTarget::Texture2D, GlTextureWrap::ClampToEdge);
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
gl.pixel_store_unpack_alignment(GlPixelStoreAlignment::Four);
```

When drawing textured geometry, set the sampler uniform with a typed uniform
location:

```rust,ignore
let font_location = gl.required_uniform_location(program, "u_font")?;
gl.uniform_1i(font_location, 0);
```

## Optional Features

Use capability helpers when a fallback path needs to choose between features:

```rust,ignore
if gl.supports_shader_pipeline() {
    // Build a shader/buffer renderer.
} else {
    // Keep a clear-only diagnostic frame visible.
}

if gl.supports_textures() {
    // Add bitmap text or sprites.
}

if gl.supports_vertex_arrays() {
    // Use VAOs for core-profile desktop GL.
}
```

The compatibility diagnostic example follows this shape: clear first, then
triangle, then text. Each layer can fail without hiding the earlier visible
output.

## Destroy

Delete GL-owned objects in `hw_context_destroy` or from `unload_game` while a
valid context is still available:

```rust,ignore
fn hw_context_destroy(&mut self, _runtime: &mut Runtime<'_>) {
    if let Some(gl) = self.gl.as_ref() {
        if let Some(program) = self.program.take() {
            gl.delete_program(program);
        }
    }

    self.gl = None;
}
```

It is fine to call cleanup methods even when an optional feature is unavailable.
For example, `delete_texture`, `unbind_texture`, `disable`, and `use_no_program`
are no-ops if that symbol group was not loaded.

## Troubleshooting

Use visible diagnostics for frontend compatibility work. The
`retrocompat-libretro` example uses staged GL initialization, clear-color
failure states, frontend messages, text overlays, and legal duplicate-frame
fallbacks after hardware mode is accepted.

Useful debugging sequence:

1. Clear the frontend FBO and submit a hardware frame.
2. Add shader and buffer setup for one triangle.
3. Add texture upload only after the triangle path is stable.
4. Add text or richer renderer state last.
5. On failure, show a frontend message and keep the last simpler visible path.

Reference: [Hardware Rendering and OpenGL](api/hardware-opengl.md).
