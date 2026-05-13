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
