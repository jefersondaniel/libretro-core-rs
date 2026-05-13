# Modern OpenGL Core

Source: [`examples/demo-libretro`](https://github.com/jefersondaniel/libretro-core-rs/blob/main/examples/demo-libretro/src/lib.rs)

The demo example is a hardware-rendered core with input and audio feedback. It
shows:

- content/no-game handling,
- modern preferred OpenGL context negotiation,
- typed GL symbol loading with `glsym::init`,
- shader/program/buffer/vertex-array setup,
- typed joypad polling,
- generated audio mixed into a silent frame batch,
- hardware frame submission with `video_refresh_hw_with_audio`,
- fallback duplicate-frame submission when hardware state is unavailable.

The triangle changes color based on whether content was supplied and moves with
joypad input. This makes it a useful smoke test for content loading, input,
OpenGL, and audio pacing in one core.
