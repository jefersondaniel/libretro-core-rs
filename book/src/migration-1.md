# Migrating to 1.0

Version 1.0 intentionally removes the bespoke typed OpenGL API. Libretro callbacks,
input/audio services and hardware negotiation remain; rendering uses re-exported
glow 0.17. The companion diagnostics crate also has a breaking 1.0 API.

```toml
libretro = { package = "libretro-core", version = "1" }
```

| Before | Now |
| --- | --- |
| `Gl::init(runtime)` | `runtime.create_glow_context()` inside `hw_context_reset` |
| `libretro::Gl`, `glsym`, `CompatGl*` | `libretro::glow::Context` |
| `GlBuffer`, `GlTexture`, other typed handles | Standard glow associated/native handle types |
| Custom target/filter/capability enums | Standard `glow::*` constants |
| Safe wrapper draw/upload calls | Standard unsafe glow methods with explicit caller safety obligations |
| `gen_buffer`, generic `buffer_data<T>` | `create_buffer`, `buffer_data_u8_slice` with initialized bytes |
| `GlFramebuffer::from_raw(0)` | `None` for the default framebuffer; nonzero native names use `NativeFramebuffer` |
| Clone/Copy dispatch tables | Own a glow context; borrow it for rendering |
| `StagedDiagnosticGl`, `DiagnosticGl`, `DiagnosticFrameText`, `render_diagnostic_gl_frame` | Standard glow initialization; frontend messages/duplicate frames on failure; `DiagnosticTextOverlay` for successful initialization |
| `DiagnosticTextOverlay::new(&gl, &gl, lines)` | `unsafe { DiagnosticTextOverlay::new(&gl, lines) }` |
| `overlay.destroy(&gl, &gl)` | `unsafe { overlay.destroy(&gl) }`, consuming the overlay |
| Public fake GL types and generated GL registry/audit scripts | Removed; consumer tests should exercise their renderer or real GL integration |

There is no legacy compatibility feature. Rewrite custom enums and helpers at the
call site; do not carry the old dispatch layer into your application.

```rust
use libretro::glow::{self, HasContext};
// In hw_context_reset:
// let gl = runtime.create_glow_context()?;
// Store gl and pass &gl to your existing glow renderer.
```

Rebuild GPU resources after reset. Delete them in `hw_context_destroy`, never
through a newly reset context. Glow does not automatically restore frontend GL
state or validate memory transfers. Read the [lifecycle tutorial](opengl.md)
before translating previously safe-looking calls into unsafe glow calls.

GLES2 and GLES3 remain supported. Partial-symbol clear-only initialization is
removed: glow requires a valid version/extension query path. On initialization
failure, show a frontend message and duplicate the prior hardware frame. Software
pixels remain legal only when hardware mode was not accepted.

Multi-platform applications can depend directly on glow in their shared renderer;
only their libretro host needs this crate. Use the same glow 0.17 dependency line
to keep Context types identical. No game-engine dependency is introduced here.
