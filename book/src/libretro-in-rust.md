# Libretro In Rust

Libretro is a frontend/core boundary. The frontend loads a dynamic library,
installs callbacks for video, audio, input, and environment commands, then calls
the core one video frame at a time.

In C, a core implements many `retro_*` symbols. In `libretro-core-rs`, you
implement `Core` and let `export_core!` provide those symbols.

## What You Implement

The game, emulator, or app code defines its own struct. That struct is the
core's state: framebuffer memory, loaded content, emulation state, GL handles,
audio buffers, input state, timers, diagnostics, and anything else that must
live across frames.

```rust,ignore
struct MyCore {
    frame: Vec<u16>,
    silence: Vec<[i16; 2]>,
    frame_index: u64,
}
```

The library does not provide a concrete core struct for you to fill in. Instead
it provides the `Core` trait. Your struct implements that trait:

```rust,ignore
impl Core for MyCore {
    fn system_info(&self) -> SystemInfo {
        SystemInfo::new("my-core", "0.1.0")
    }

    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(320, 240, 60.0, 48_000.0)
    }

    fn run(&mut self, runtime: &mut Runtime<'_>) {
        runtime.poll_input();
        self.frame_index = self.frame_index.wrapping_add(1);
    }
}
```

`Core` is a trait because every core has different state and behavior. The
trait is the typed Rust contract that replaces hand-written `retro_*` ABI
functions.

## What The Library Owns

The library owns the libretro ABI boundary. `libretro::export_core!` generates
the exported C symbols that frontends look for, stores your core instance, calls
your trait methods at the right time, converts raw frontend callbacks into typed
Rust handles, and catches panics before they cross the ABI boundary.

```rust,ignore
libretro::export_core!(MyCore::default());
```

The frontend still controls when methods are called. Your code controls what
happens inside those methods.

## Runtime

`Runtime` is a short-lived handle passed by the library into methods that can
interact with the running frontend. Most cores first meet it in `load_game` and
`run`:

```rust,ignore
fn load_game(&mut self, game: Option<GameInfo<'_>>, runtime: &mut Runtime<'_>) -> bool {
    runtime.logger().info("loading content");
    game.is_some()
}

fn run(&mut self, runtime: &mut Runtime<'_>) {
    runtime.poll_input();
    let _ = runtime.video_refresh_frame_with_audio(
        &self.frame,
        320,
        240,
        320 * core::mem::size_of::<u16>(),
        &self.silence,
    );
}
```

`Runtime` is not a game engine runtime and it is not global application state.
It is the typed access point for frontend services that are valid during the
current callback. Use your core struct for persistent state; use `Runtime` to
talk to the frontend.

Common `Runtime` jobs:

- Poll input before reading controller, keyboard, mouse, pointer, or lightgun
  state.
- Submit software frames, hardware frames, duplicate frames, and audio batches.
- Query the current hardware framebuffer after hardware rendering is active.
- Send frontend messages and logging output.
- Access environment commands that are valid at runtime.
- Use optional frontend services such as rumble, LEDs, sensors, camera,
  microphone, MIDI, VFS, performance counters, and netplay helpers.

| Libretro concept | Rust API |
| --- | --- |
| Core metadata | `Core::system_info`, `SystemInfo`, `ContentContract` |
| Environment callback | `Core::on_set_environment`, `Environment` |
| Content loading | `Core::load_game`, `GameInfo` |
| AV information | `Core::av_info`, `SystemAvInfo` |
| Per-frame execution | `Core::run`, `Runtime` |
| Hardware context reset | `Core::hw_context_reset`, `Runtime::create_glow_context` |
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
