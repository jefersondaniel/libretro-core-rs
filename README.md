# libretro-core-rs

Rust helpers for building [libretro](https://www.libretro.com/) cores.

`libretro-core` wraps the libretro C ABI behind a single `Core` trait, so you
write idiomatic Rust and the crate handles the `retro_*` symbol exports, the
environment callback, AV info negotiation, and content registration for you.

## Features

- **Safe-ish `Core` trait.** Implement a handful of methods (`system_info`,
  `av_info`, `run`, …) instead of dozens of `unsafe extern "C"` functions.
- **One-line symbol export.** `libretro::export_core!(MyCore::default())`
  generates every `retro_*` entry point the frontend looks for.
- **AV helpers.** `fixed_system_av_info`, `silent_stereo_frames_for_video_frame`,
  and pixel-format helpers cover the boilerplate around `SystemAvInfo` and
  audio/video pacing.
- **Content contracts.** Declare supported extensions and "no content required"
  mode with `ContentContract`; the crate handles the matching environment
  callbacks.
- **Hardware rendering.** Negotiate an OpenGL/GLES context with the frontend
  and load GL entry points through the libretro callback path (no direct
  linking against `libGL`).
- **Input + runtime.** A `Runtime` handle exposes input polling, video refresh,
  audio submission, and environment access during `run`.

## Quick start

Add the dependency and configure your crate as a `cdylib`:

```toml
# Cargo.toml
[package]
name = "hello-libretro"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
libretro = { package = "libretro-core", version = "0.1" }
```

A complete "hello world" core that paints a blue framebuffer at 320×240 / 60 Hz
with silent stereo audio:

```rust
// src/lib.rs
use libretro::{
    ContentContract, Core, Environment, GameInfo, Runtime, SystemAvInfo, SystemInfo,
    fixed_system_av_info, silent_stereo_frames_for_video_frame,
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const FPS: u32 = 60;
const SAMPLE_RATE: u32 = 48_000;
const BLUE_0RGB1555: u16 = 0x001F;

#[derive(Default)]
struct HelloCore;

impl Core for HelloCore {
    fn system_info(&self) -> SystemInfo {
        SystemInfo::new("hello-libretro", env!("CARGO_PKG_VERSION"))
    }

    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(WIDTH, HEIGHT, FPS as f64, SAMPLE_RATE as f64)
    }

    fn on_set_environment(&mut self, env: &mut Environment<'_>) {
        // Tell the frontend this core can start without a ROM.
        let _ = ContentContract::new("")
            .with_support_no_game(true)
            .register_environment(env);
    }

    fn load_game(&mut self, _game: Option<GameInfo<'_>>, _rt: &mut Runtime<'_>) -> bool {
        true
    }

    fn run(&mut self, rt: &mut Runtime<'_>) {
        rt.poll_input();

        let frame = vec![BLUE_0RGB1555; (WIDTH * HEIGHT) as usize];
        let audio = silent_stereo_frames_for_video_frame(SAMPLE_RATE, FPS);

        let _ = rt.video_refresh_frame_with_audio(
            &frame,
            WIDTH,
            HEIGHT,
            WIDTH as usize * std::mem::size_of::<u16>(),
            &audio,
        );
    }
}

libretro::export_core!(HelloCore::default());
```

Build it:

```sh
cargo build --release
```

The resulting `target/release/libhello_libretro.so` (or `.dylib` / `.dll`) can
be loaded by any libretro frontend such as RetroArch.

## Workspace layout

| Crate | Purpose |
|---|---|
| [`libretro-core`](crates/libretro-core) | The library above — safe `Core` trait, AV/content helpers, hardware-render negotiation, `glsym` GL symbol access. |
| [`libretro-diagnostics`](crates/libretro-diagnostics) | Optional GL/text/frame helpers for cores that need on-screen failure output. |
| [`examples/software-libretro`](examples/software-libretro) | Minimal software framebuffer core. |
| [`examples/retrocompat-libretro`](examples/retrocompat-libretro) | OpenGL/GLES compatibility triangle and text diagnostic core. |
| [`examples/demo-libretro`](examples/demo-libretro) | Generic OpenGL/input/audio demo core. |

Run the test suite with:

```sh
cargo test --workspace
```

## License

MIT — see [LICENSE](LICENSE).
