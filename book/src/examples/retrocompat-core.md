# Compatibility OpenGL Core

Source: [`examples/retrocompat-libretro`](https://github.com/jefersondaniel/libretro-core-rs/blob/main/examples/retrocompat-libretro/src/lib.rs)

The compatibility example targets a conservative OpenGL path and emphasizes
diagnosability. It shows:

- compatibility hardware-render candidates,
- staged GL initialization,
- visible software fallback when hardware negotiation fails,
- distinct clear colors for initialization failures,
- diagnostic text overlays,
- performance sampling and display,
- GL cleanup on unload/reset.

Use this example when working on frontend compatibility or when adding
diagnostic behavior to a hardware core. It demonstrates the project rule that
failures should be visible and actionable instead of producing a black frame.

Failure-mode map:

- If hardware negotiation is rejected, the core returns `true` from
  `load_game`, shows a frontend message, and presents a software diagnostic
  frame.
- If hardware mode is accepted but no framebuffer is available, it shows a
  message and submits a duplicate frame with audio.
- If only clear symbols load, it still presents a clear-only hardware frame.
- If triangle rendering works but text setup fails, it keeps the triangle path
  alive and disables only the text overlay.
- If text rendering later fails, it destroys the text overlay and continues.

`libretro-diagnostics` provides the staged GL helpers and text/frame diagnostic
building blocks used by this example.

Tutorial: [OpenGL](../opengl.md).
