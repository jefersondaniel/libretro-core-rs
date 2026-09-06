# Compatibility OpenGL Core

The [source](https://github.com/jefersondaniel/libretro-core-rs/blob/main/examples/retrocompat-libretro/src/lib.rs)
requests compatibility GL/GLES2 and obtains a standard glow context during reset.
It draws a rotating triangle and an embedded-font performance overlay. The shared
example renderer also selects GLES3 or desktop shader syntax from the live version.

Hardware negotiation rejection produces a red software diagnostic. After hardware
acceptance, initialization failures use frontend messages and duplicate frames.
If glow works but shader setup fails, a magenta clear frame remains available.
Text setup failure does not discard a working triangle renderer.

Resources are deleted during context destroy and abandoned on replacement reset
or unload. No GL calls occur from ordinary Drop. Performance reporting includes
frame interval, render submission, video refresh, audio, draws and uploads.

Build with `cargo build -p retrocompat-libretro` and load without content.
Start with the [complete glow tutorial](../opengl.md) for the minimal lifecycle.
