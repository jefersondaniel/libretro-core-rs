# Introduction

`libretro-core-rs` is a Rust workspace for writing libretro cores without
making core authors work directly against the C ABI.

A libretro core is a dynamic library loaded by a frontend such as RetroArch.
The frontend owns the window, audio device, input mapping, and optional OpenGL
context. The core owns the emulation or application state, reports metadata and
AV timing, accepts content, and produces one frame of video and audio whenever
the frontend calls it.

The public API is intentionally Rust-first. Normal core code should use enums,
newtypes, builders, slices, owned strings, `Option`, and `Result` instead of raw
callback tables, magic numbers, unchecked pointers, or hand-written
`retro_*` exports.

The project has two library crates:

- `libretro-core`, imported as `libretro`, owns the `Core` trait,
  `export_core!` macro, typed libretro wrappers, runtime helpers, input
  polling, callback event routing, environment commands, and OpenGL symbol
  wrappers.
- `libretro-diagnostics` owns optional visible diagnostics for cores that want
  clear failure frames instead of silent black screens.

Read the tutorial chapters in order if you are starting a core from scratch.
Use the API reference when you already know the libretro concept and need exact
entry points.

Useful local references:

- [Libretro In Rust](libretro-in-rust.md)
- [Hello World Core](hello-world-core.md)
- [Rustdoc Map](reference/rustdoc-map.md)
- [Local Libretro Specs](reference/libretro-specs.md)
