# Software Core

Source: [`examples/software-libretro`](https://github.com/jefersondaniel/libretro-core-rs/blob/main/examples/software-libretro/src/lib.rs)

The software example is the smallest complete core in the workspace. It shows:

- `SystemInfo` and `ContentContract` setup,
- fixed geometry and timing through `fixed_system_av_info`,
- no-game support,
- one software framebuffer format,
- one audio batch per video frame,
- `runtime.poll_input()` before frame submission.

The frame is a constant blue 0RGB1555 buffer. The audio path uses
`silent_stereo_frames_for_video_frame(48_000, 60)` so the number of frames per
video frame is derived from the same timing contract reported to the frontend.

Use this example when validating lifecycle behavior before adding OpenGL,
options, or advanced frontend services.

Copy this pattern for the first reusable software core:

- keep the framebuffer and audio batch in core state,
- use one `content_contract()` helper for `system_info` and
  `on_set_environment`,
- call `runtime.poll_input()` at the start of `run`,
- submit with `video_refresh_frame_with_audio`.

Tutorials:

- [Hello World Core](../hello-world-core.md)
- [Frame Loop Basics](../frame-loop-basics.md)
- [Audio](../audio.md)
