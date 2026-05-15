# Hello World Core

This chapter builds the first core: a no-content dynamic library that paints a
blue 320x240 software frame at 60 FPS and submits silent stereo audio at
48 kHz.

## Cargo Setup

Configure the crate as a `cdylib`. A libretro frontend loads a dynamic library;
`cargo run` is not the execution model.

```toml
[package]
name = "hello-libretro"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
libretro = { package = "libretro-core", version = "0.1" }
```

Inside this workspace, the example uses a path dependency instead:

```toml
libretro = { package = "libretro-core", path = "../../crates/libretro-core" }
```

## Core Code

The reusable shape is the same as `examples/software-libretro`: keep frame and
audio buffers in core state, use one content contract helper, and submit one
video frame plus one audio batch from `run`.

```rust,ignore
use libretro::{
    ContentContract, Core, Environment, GameInfo, Runtime, SystemAvInfo,
    SystemInfo, fixed_system_av_info, silent_stereo_frames_for_video_frame,
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const FPS_HZ: u32 = 60;
const SAMPLE_RATE_HZ: u32 = 48_000;
const BLUE_0RGB1555: u16 = 0x001f;

struct HelloCore {
    frame: Vec<u16>,
    silence: Vec<[i16; 2]>,
}

impl Default for HelloCore {
    fn default() -> Self {
        Self {
            frame: vec![BLUE_0RGB1555; (WIDTH * HEIGHT) as usize],
            silence: silent_stereo_frames_for_video_frame(SAMPLE_RATE_HZ, FPS_HZ),
        }
    }
}

impl Core for HelloCore {
    fn system_info(&self) -> SystemInfo {
        let mut info = SystemInfo::new("hello-libretro", env!("CARGO_PKG_VERSION"));
        content_contract().apply_to_system_info(&mut info);
        info
    }

    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(WIDTH, HEIGHT, FPS_HZ as f64, SAMPLE_RATE_HZ as f64)
    }

    fn on_set_environment(&mut self, env: &mut Environment<'_>) {
        let _ = content_contract().register_environment(env);
    }

    fn load_game(&mut self, _game: Option<GameInfo<'_>>, _runtime: &mut Runtime<'_>) -> bool {
        true
    }

    fn run(&mut self, runtime: &mut Runtime<'_>) {
        runtime.poll_input();
        let pitch = WIDTH as usize * core::mem::size_of::<u16>();
        let _ = runtime.video_refresh_frame_with_audio(
            &self.frame,
            WIDTH,
            HEIGHT,
            pitch,
            &self.silence,
        );
    }
}

fn content_contract() -> ContentContract {
    ContentContract::new("").with_support_no_game(true)
}

libretro::export_core!(HelloCore::default());
```

`ContentContract::new("").with_support_no_game(true)` describes a core that can
start without content. Required-content cores use extensions such as
`"bin|dat"` and return `false` from `load_game` when required content is absent
or invalid.

The software frame uses libretro's default 0RGB1555 format. That keeps hello
world small, but a real software renderer may choose another pixel format
explicitly during `load_game`.

Audio batches are `&[[i16; 2]]`: one element is one stereo frame, left then
right. At 48,000 Hz and 60 FPS, this core submits 800 silent stereo frames per
video frame.

## Build And Load

Build the core:

```sh
cargo build --release
```

On Linux, the output for the example is a dynamic library such as
`target/release/libhello_libretro.so`. Other platforms use their normal dynamic
library suffixes, such as `.dylib` or `.dll`.

Load the library in a libretro frontend without content. The expected result is
a solid blue 320x240 frame and silence.
