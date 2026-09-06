# Modern OpenGL Core

Source: [`examples/demo-libretro`](https://github.com/jefersondaniel/libretro-core-rs/blob/main/examples/demo-libretro/src/lib.rs)

The demo example is a hardware-rendered core with input and audio feedback. It
shows:

- content/no-game handling,
- modern preferred OpenGL context negotiation,
- standard glow loading with `Runtime::create_glow_context`,
- shader/program/buffer/vertex-array setup,
- typed joypad polling,
- embedded WAV audio mixed into a silent frame batch,
- hardware frame submission with `video_refresh_hw` and `audio_sample_batch`,
- fallback duplicate-frame submission when hardware state is unavailable.

The triangle changes color based on whether content was supplied and moves with
joypad input. This makes it a useful smoke test for content loading, input,
OpenGL, and audio pacing in one core.

Lifecycle map:

- `load_game` logs optional content, sets `PixelFormat::Xrgb8888`, and requests
  `opengl_modern_preferred_hw_render_candidates()`.
- `run` polls joypad input, mixes a short sound effect into a silent audio
  batch, renders into `runtime.current_framebuffer()`, and submits
  `video_refresh_hw` and `audio_sample_batch`.
- `hw_context_reset` loads `glow::Context`, picks shader sources for the live GL
  family, and creates the program, buffer, and optional vertex array.
- `hw_context_destroy` deletes GL-owned objects and clears cached handles.

Fallbacks in this demo keep audio moving with duplicate frames. For visible
OpenGL diagnostics, use the compatibility example.

Tutorials:

- [Input](../input.md)
- [Audio](../audio.md)
- [OpenGL](../opengl.md)
