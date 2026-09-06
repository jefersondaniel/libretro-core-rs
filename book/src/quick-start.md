# Quick Start

This page is the route map for creating a libretro core with this workspace.
Read it first, then follow the links for details when a concept becomes
relevant.

## Concept Overview

| Concept | What it means | Where to learn more |
| --- | --- | --- |
| Core struct | Your persistent state: content, framebuffer, audio buffers, renderer handles, counters, and emulator/game state. The library does not provide this struct. | [Core Lifecycle](api/core-lifecycle.md) |
| `Core` trait | The Rust trait your struct implements. It replaces hand-written `retro_*` functions. | [Libretro In Rust](libretro-in-rust.md) |
| `export_core!` | The macro that exports the libretro ABI symbols and dispatches frontend calls to your `Core` implementation. | [Raw ABI Boundaries](api/raw-abi.md) |
| `SystemInfo` | Metadata such as core name, version, and supported content extensions. | [Content, AV, and Timing](api/content-av-timing.md) |
| `ContentContract` | Reusable declaration of content extensions and whether the core can start without content. | [Content, AV, and Timing](api/content-av-timing.md) |
| `Environment` | Setup and frontend-negotiation commands: content support, pixel format, options, controller info, hardware rendering, messages, services. | [Environment and Core Options](api/environment-options.md) |
| `Runtime` | Temporary handle passed into lifecycle methods so your core can poll input, submit frames/audio, log messages, and access runtime frontend services. Do not store it. | [Runtime Video and Audio](api/runtime-video-audio.md) |
| `SystemAvInfo` | Video geometry, FPS, and audio sample rate reported to the frontend. | [Content, AV, and Timing](api/content-av-timing.md) |
| Frame loop | The work done in `Core::run`: poll input, advance one frame of state, submit video and audio. | [Frame Loop Basics](frame-loop-basics.md) |
| Hardware rendering | Optional OpenGL/GLES context negotiation and standard glow rendering. | [OpenGL](opengl.md) |

## Step By Step

1. Create a Rust library crate and configure it as a `cdylib`.
   A libretro frontend loads a dynamic library, not a normal executable.
   See [Hello World Core](hello-world-core.md) and [Running A Core](running-a-core.md).

2. Define your core state struct.
   Put persistent data here: framebuffers, loaded content, emulation state,
   audio scratch buffers, GL handles, diagnostics, and counters.
   See [Core Lifecycle](api/core-lifecycle.md).

3. Implement the `Core` trait for your struct.
   Start with `system_info`, `av_info`, `load_game`, and `run`.
   Add optional lifecycle methods only when needed.
   See [Libretro In Rust](libretro-in-rust.md).

4. Describe content support with `ContentContract`.
   Use it in both `system_info` and `on_set_environment` so metadata and
   frontend negotiation stay consistent.
   See [Content, AV, and Timing](api/content-av-timing.md).

5. Report video and audio timing with `SystemAvInfo`.
   Most fixed-size cores can use `fixed_system_av_info(width, height, fps,
   sample_rate)`.
   See [Audio](audio.md) for sample pacing.

6. Use `Environment` during setup.
   Register content support, input descriptors, controller info, core options,
   pixel format, hardware rendering, or frontend services from
   `on_set_environment` and `load_game`.
   See [Environment and Core Options](api/environment-options.md).

7. Accept or reject content in `load_game`.
   Inspect `GameInfo` if content was supplied, initialize core state, and return
   `true` only when the core can run.
   See [Hello World Core](hello-world-core.md) for no-content startup and
   [Software Core](examples/software-core.md) for a reusable minimal lifecycle.

8. Write the frame loop in `run`.
   Use the supplied `Runtime`: call `poll_input`, read input, advance one frame,
   and submit video plus audio. Keep `Runtime` out of your core struct.
   See [Runtime Video and Audio](api/runtime-video-audio.md),
   [Frame Loop Basics](frame-loop-basics.md), and [Input](input.md).

9. Export the core.
   End the library with `libretro::export_core!(MyCore::default())` or another
   constructor that creates the initial core state.
   See [Raw ABI Boundaries](api/raw-abi.md).

10. Validate and run it.
    Use `cargo test --workspace` in this repo, build the core as a dynamic
    library, then load it in a frontend.
    See [Running A Core](running-a-core.md) and [Publishing and Validation](reference/publishing.md).

11. Add optional systems as the core grows.
    Input descriptors, core options, memory maps, disks, save states, VFS,
    sensors, camera, microphone, MIDI, performance counters, and OpenGL all have
    focused chapters in the API reference.

The canonical minimal software pattern is `examples/software-libretro`. It
keeps the framebuffer and silent audio batch in core state so `run` can avoid
per-frame allocation:

```rust,ignore
use libretro::{
    ContentContract, Core, Environment, GameInfo, Runtime, SystemAvInfo,
    SystemInfo, fixed_system_av_info, silent_stereo_frames_for_video_frame,
};

struct MyCore {
    frame: Vec<u16>,
    silence: Vec<[i16; 2]>,
}

impl Default for MyCore {
    fn default() -> Self {
        Self {
            frame: vec![0x001f; 320 * 240],
            silence: silent_stereo_frames_for_video_frame(48_000, 60),
        }
    }
}

impl Core for MyCore {
    fn system_info(&self) -> SystemInfo {
        let mut info = SystemInfo::new("my-core", "0.1.0");
        ContentContract::new("bin")
            .with_support_no_game(true)
            .apply_to_system_info(&mut info);
        info
    }

    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(320, 240, 60.0, 48_000.0)
    }

    fn on_set_environment(&mut self, env: &mut Environment<'_>) {
        let _ = ContentContract::new("bin")
            .with_support_no_game(true)
            .register_environment(env);
    }

    fn load_game(&mut self, _game: Option<GameInfo<'_>>, _runtime: &mut Runtime<'_>) -> bool {
        true
    }

    fn run(&mut self, runtime: &mut Runtime<'_>) {
        runtime.poll_input();
        let pitch = 320 * core::mem::size_of::<u16>();
        let _ = runtime.video_refresh_frame_with_audio(
            &self.frame,
            320,
            240,
            pitch,
            &self.silence,
        );
    }
}

libretro::export_core!(MyCore::default());
```

Use the [hello-world tutorial](hello-world-core.md) for the first buildable
core, the [software example](examples/software-core.md) for the smallest
complete reusable lifecycle, and the [modern OpenGL example](examples/demo-core.md)
when you need hardware rendering.
