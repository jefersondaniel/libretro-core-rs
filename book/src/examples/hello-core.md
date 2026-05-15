# Hello World Core

Source: [`examples/hello-libretro`](https://github.com/jefersondaniel/libretro-core-rs/blob/main/examples/hello-libretro/src/lib.rs)

The hello example is the first smoke-test core. It demonstrates:

- a `cdylib` package,
- `Core` plus `export_core!`,
- no-game startup,
- fixed 320x240 / 60 FPS / 48 kHz AV info,
- a solid blue default-format software frame,
- silent stereo audio.

The source intentionally stays tiny. For the reusable minimal lifecycle, use
[`examples/software-libretro`](software-core.md), which keeps frame/audio buffers
in core state and applies its content contract consistently to metadata and the
environment.

Tutorial: [Hello World Core](../hello-world-core.md).
