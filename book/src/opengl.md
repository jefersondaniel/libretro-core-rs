# OpenGL

Libretro hardware rendering is a frontend-owned OpenGL or OpenGL ES context.
The core negotiates a context, initializes GL state when the frontend creates
the context, renders into the frontend-provided framebuffer each frame, and
deletes GL-owned objects before the context is gone.

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

## Reset

Load symbols and rebuild context-owned objects in `hw_context_reset`:

```rust,ignore
fn hw_context_reset(&mut self, runtime: &mut Runtime<'_>) {
    let gl = glsym::init(runtime)
        .unwrap_or_else(|error| panic!("failed to load OpenGL symbols: {error}"));

    let program = gl
        .build_program(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE)
        .unwrap_or_else(|error| panic!("failed to build GL program: {error}"));

    self.gl = Some(gl);
    self.program = Some(program);
}
```

The frontend may recreate the context. Treat the symbol table, programs,
buffers, textures, vertex arrays, and framebuffer-dependent state as
context-lifetime state.

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

Set the viewport every frame. Do not leave shared GL state such as framebuffers,
buffers, textures, vertex arrays, or programs bound when handing the frame back
to the frontend.

After hardware rendering is active, fallback frames should be hardware or
duplicate submissions with audio. Software-pixel fallback is only appropriate
before hardware negotiation succeeds.

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

## Troubleshooting

Use visible diagnostics for frontend compatibility work. The
`retrocompat-libretro` example uses staged GL initialization, clear-color
failure states, frontend messages, text overlays, and legal duplicate-frame
fallbacks after hardware mode is accepted.

Reference: [Hardware Rendering and OpenGL](api/hardware-opengl.md).
