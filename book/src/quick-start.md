# Quick Start

Use this page as a route map. The full path starts with
[Libretro In Rust](libretro-in-rust.md), builds the
[Hello World Core](hello-world-core.md), then explains running, frame pacing,
input, audio, and OpenGL.

The compact lifecycle is:

1. Configure the crate as a `cdylib`.
2. Implement `Core`.
3. Return stable `SystemInfo` and `SystemAvInfo`.
4. Register the same `ContentContract` with `SystemInfo` and `Environment`.
5. Accept or reject content in `load_game`.
6. In `run`, call `runtime.poll_input()`, update state, and submit video/audio.
7. Export the core with `libretro::export_core!`.

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
