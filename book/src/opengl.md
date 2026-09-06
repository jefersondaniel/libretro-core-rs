# OpenGL with glow

`libretro-core` negotiates a frontend-owned context and gives you a standard
`glow::Context`. You write ordinary glow rendering code; no custom GL types,
manual symbol casts, or separate glow dependency are needed.

```toml
[dependencies]
libretro = { package = "libretro-core", version = "1" }
```

The default `glow` feature provides both `Runtime::create_glow_context()` and
`libretro::glow`. Software-only cores can use `default-features = false`.
The loader is for native libretro hardware contexts, not browser WebGL contexts.

## Complete first core

This example is compiled as part of the workspace. It requests desktop GL or
GLES2, clears the current frontend framebuffer, polls input, and submits audio.

```rust
{{#include ../../examples/glow-libretro/src/lib.rs}}
```

Build with `cargo build -p glow-libretro`. Load the resulting shared library in
RetroArch without content. See [Running a Core](running-a-core.md).

## Lifecycle

| Callback | Responsibility |
| --- | --- |
| `load_game` | Negotiate pixel format and hardware context; do not call the glow constructor yet. |
| `hw_context_reset` | Call `create_glow_context`, then create GPU resources. Rebuild after every reset. |
| `run` | Query the current framebuffer, bind it, render, release your bindings, submit hardware frame and audio. |
| `hw_context_destroy` | Delete resources while the retiring context is current; discard glow. |
| `unload_game` / ordinary `Drop` | Abandon remaining handles without GL calls. |

The returned object does not own the frontend context. Do not use it from other
threads or callbacks without a current-context guarantee. Glow commands are
unsafe: resource lifetime, buffer size, format and inherited GL state remain your
responsibility. In particular, reset unpack row/skip settings and unbind pixel
unpack buffers before uploading Rust slices when those features are supported.

Avoid registering glow debug callbacks unless you can unregister them while the
same context remains current: glow's context destructor may call GL to unregister
an installed callback. The examples install none.

## Errors and capabilities

The constructor rejects calls outside reset and non-GL contexts. Missing startup
symbols and initialization panics become errors on unwind builds; panic-abort
builds cannot recover a glow initialization panic. Invalid frontend procedure
addresses cannot be made safe by a loader.

`gl.version()` and `gl.supported_extensions()` describe the live context. A loaded
function pointer does not prove a feature is supported. GLES2 is supported; GLES3
operations must be gated. The adapter resolves advertised ARB/EXT/ANGLE divisor
aliases; it is not an emulator for unavailable GL features.

If initialization fails after hardware negotiation, log/show a frontend message
and submit duplicate frames with audio. Do not submit software pixels in hardware
mode. The old partial-symbol clear-only dispatcher has been removed.

Continue with the [hardware API reference](api/hardware-opengl.md),
[triangle demo](examples/demo-core.md), or [migration guide](migration-1.md).
