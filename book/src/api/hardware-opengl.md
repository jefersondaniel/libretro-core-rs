# Hardware Rendering and OpenGL

The frontend owns the context and render target. `Environment` negotiates that
contract; `Runtime` supplies a glow command interface and frame submission.
Rendering code can accept `&glow::Context` and remain independent of libretro.

Start with the [complete glow tutorial](../opengl.md). The methods below are the
library's integration surface; GL commands themselves belong to glow.

| Method / helper | Description |
| --- | --- |
| `Environment::set_hw_render` | Request one typed hardware context configuration. |
| `Environment::set_hw_render_from_candidates` | Negotiate an ordered candidate list, respecting frontend preferences. |
| `opengl_compatibility_hw_render_candidates` | Desktop compatibility GL, explicit GLES 2.0, and legacy GLES2 candidates. |
| `opengl_modern_preferred_hw_render_candidates` | The compatibility candidates plus desktop core 3.3 and GLES3. |
| `Runtime::create_glow_context` | Return `Result<glow::Context, String>` during context reset; available with the default `glow` feature on native targets. |
| `Runtime::hw_context_type` | Inspect the negotiated libretro context family. |
| `Runtime::hw_proc_address` | Low-level frontend/process procedure lookup for advanced integrations. |
| `Runtime::current_framebuffer` | Query the target every frame; `Some(0)` is valid. |
| `Runtime::video_refresh_hw` | Submit a completed hardware frame. |
| `Runtime::video_refresh_hw_with_audio` | Submit a hardware frame and interleaved stereo samples. |
| `Runtime::video_refresh_dupe_with_audio` | Duplicate the previous frame and continue audio on a failure path. |

## Configuration and enums

`HwRenderConfig` describes the requested family, version, depth/stencil needs,
origin, caching, and debug preference. `HwContextType` distinguishes desktop GL,
core GL, GLES2, GLES3, and explicitly versioned GLES. Context negotiation does not
promise every extension; check the live glow version and extension set.

## Standard glow commands

Import the trait to bring GL methods into scope:

```rust
use libretro::glow::{self, HasContext};
```

| Surface | Common methods | Contract |
| --- | --- | --- |
| Context facts | `version`, `supported_extensions` | Inspect cached capabilities. |
| Programs | `create_shader`, `compile_shader`, `link_program`, `use_program` | Check compile/link status and logs; delete resources explicitly. |
| Buffers | `create_buffer`, `bind_buffer`, `buffer_data_u8_slice` | Supply initialized bytes and valid binding/lifetime. |
| Textures | `create_texture`, `bind_texture`, `tex_image_2d` | Validate byte size and pixel-unpack state. |
| Drawing | `vertex_attrib_pointer_f32`, `draw_arrays`, `draw_elements` | Configure valid attributes, index ranges, and rasterization state. |
| Framebuffer | `bind_framebuffer`, `viewport`, `clear` | Bind this frame's target; represent framebuffer zero with `None`. |
| Cleanup | `delete_buffer`, `delete_texture`, `delete_program` | Call only with the owning context current. |

For the complete method tables, associated handle types, constants and safety
contracts, use the [glow 0.17 Context reference](https://docs.rs/glow/0.17.0/glow/struct.Context.html)
and [HasContext reference](https://docs.rs/glow/0.17.0/glow/trait.HasContext.html).
These are standard glow objects, not a libretro-specific wrapper or fork.
