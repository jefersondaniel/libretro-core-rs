# Core Lifecycle

Core authors implement the `Core` trait. The crate owns the raw `retro_*`
exports through `export_core!`, catches panics at ABI boundaries, and converts
frontend callbacks into typed `Runtime` and `Environment` values.

## Core Trait

`Core` is a trait, not a struct supplied by the library. You create the struct
that stores your core's state, then implement `Core` for it.

```rust,ignore
struct MyCore {
    frame: Vec<u16>,
    content_loaded: bool,
}

impl Core for MyCore {
    fn system_info(&self) -> SystemInfo {
        SystemInfo::new("my-core", "0.1.0")
    }

    fn load_game(&mut self, _game: Option<GameInfo<'_>>, _runtime: &mut Runtime<'_>) -> bool {
        self.content_loaded = true;
        true
    }

    fn run(&mut self, runtime: &mut Runtime<'_>) {
        runtime.poll_input();
    }
}
```

The struct is yours. The trait methods are the points where the libretro
frontend asks your code for metadata, content decisions, frame execution, and
cleanup.

`export_core!` connects your Rust type to libretro:

```rust,ignore
libretro::export_core!(MyCore::default());
```

After that, frontends call the generated `retro_*` symbols and the library
dispatches those calls to your `Core` implementation.

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

## Core State Vs Runtime

Keep persistent data in your core struct:

- loaded ROM or content metadata,
- emulation/game state,
- framebuffers and audio scratch buffers,
- user-visible diagnostics,
- GL objects created during `hw_context_reset`,
- counters, timers, and cached frontend decisions.

Use `Runtime` only while a callback is running. It is borrowed from the library
and should not be stored in your struct. If a frontend capability or handle must
be reused later, copy the typed value you need into your own state.

| Frontend step | Core method | Main handle |
| --- | --- | --- |
| Metadata query | `system_info` | `SystemInfo` |
| Environment setup | `on_set_environment` | `Environment` |
| Content load | `load_game` | `Runtime` |
| AV query | `av_info` | `SystemAvInfo` |
| Frame execution | `run` | `Runtime` |
| Content unload | `unload_game` | core state |
| Core shutdown | `deinit` | core state |

Keep user code on the typed side of the boundary. Raw ABI details are available
for auditing and tests, but normal cores should not need `unsafe` blocks or raw
`RETRO_ENVIRONMENT_*` command numbers.

Tutorials:

- [Libretro In Rust](../libretro-in-rust.md)
- [Hello World Core](../hello-world-core.md)
- [OpenGL](../opengl.md)
