# libretro-core-rs

Rust helpers for building libretro cores.

This workspace contains:

| Crate | Purpose |
|---|---|
| `libretro-core` | Safe-ish Rust wrapper around the libretro ABI, AV/content helpers, hardware-render negotiation, and public `glsym` OpenGL symbol access. |
| `libretro-diagnostics` | Optional diagnostic GL/text/frame helpers for cores that need visible failure output. |
| `examples/software-libretro` | Minimal software framebuffer core. |
| `examples/retrocompat-libretro` | OpenGL/GLES compatibility triangle and text diagnostic core. |
| `examples/demo-libretro` | Generic OpenGL/input/audio demo core. |

Run the test suite with:

```sh
cargo test --workspace
```
