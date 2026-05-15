# Core Lifecycle

Core authors implement the `Core` trait. The crate owns the raw `retro_*`
exports through `export_core!`, catches panics at ABI boundaries, and converts
frontend callbacks into typed `Runtime` and `Environment` values.

The usual lifecycle is:

1. `system_info` returns a stable `SystemInfo`.
2. `on_set_environment` registers content support, options, callbacks, and
   frontend capabilities.
3. `load_game` accepts or rejects content and performs runtime setup.
4. `av_info` returns fixed or dynamic geometry/timing.
5. `run` polls input, advances emulation, and submits video/audio.
6. `unload_game` and `deinit` release state owned by the core.

Hardware cores also implement `hw_context_reset` and `hw_context_destroy` so GL
objects are tied to the frontend-owned context lifetime.

The high-level pieces are:

- `Core`: the trait core authors implement.
- `Runtime`: per-frame access to input, video, audio, memory, and frontend
  services.
- `Environment`: setup-time and runtime environment commands.
- `export_core!`: exports the libretro ABI symbols.

| Frontend step | Core method | Main handle |
| --- | --- | --- |
| Metadata query | `system_info` | `SystemInfo` |
| Environment setup | `on_set_environment` | `Environment` |
| Content load | `load_game` | `Runtime` |
| AV query | `av_info` | `SystemAvInfo` |
| Frame execution | `run` | `Runtime` |
| Content unload | `unload_game` | core state |
| Core shutdown | `deinit` | `Environment` |

Keep user code on the typed side of the boundary. Raw ABI details are available
for auditing and tests, but normal cores should not need `unsafe` blocks or raw
`RETRO_ENVIRONMENT_*` command numbers.

Tutorials:

- [Libretro In Rust](../libretro-in-rust.md)
- [Hello World Core](../hello-world-core.md)
- [OpenGL](../opengl.md)
