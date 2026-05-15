# Libretro In Rust

Libretro is a frontend/core boundary. The frontend loads a dynamic library,
installs callbacks for video, audio, input, and environment commands, then calls
the core one video frame at a time.

In C, a core implements many `retro_*` symbols. In `libretro-core-rs`, you
implement `Core` and let `export_core!` provide those symbols.

| Libretro concept | Rust API |
| --- | --- |
| Core metadata | `Core::system_info`, `SystemInfo`, `ContentContract` |
| Environment callback | `Core::on_set_environment`, `Environment` |
| Content loading | `Core::load_game`, `GameInfo` |
| AV information | `Core::av_info`, `SystemAvInfo` |
| Per-frame execution | `Core::run`, `Runtime` |
| Hardware context reset | `Core::hw_context_reset`, `glsym::init` |
| Hardware context destroy | `Core::hw_context_destroy` |
| ABI symbol exports | `libretro::export_core!` |

`Environment` is available during setup and through `Runtime::environment()`.
Use it for frontend negotiation: content support, pixel format, hardware
rendering, input descriptors, controller info, core options, messages, and
optional services.

`Runtime` is the per-frame handle. Use it to poll input, submit video and
audio, query hardware framebuffers, show messages, access logging, and use
frontend services that are valid while the core is running.

The normal lifecycle is:

1. The frontend asks for `system_info`.
2. The frontend supplies the environment callback, which dispatches
   `on_set_environment`.
3. The frontend calls `load_game`.
4. The frontend asks for `av_info`.
5. The frontend repeatedly calls `run`.
6. The frontend later calls `unload_game` and `deinit`.

Hardware-rendered cores add `hw_context_reset` and `hw_context_destroy`. The
frontend owns the context and may recreate it, so GL symbols and GL object
handles are context-lifetime state.
