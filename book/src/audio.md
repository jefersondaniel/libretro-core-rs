# Audio

Libretro audio uses signed 16-bit stereo frames. In this crate, a batch is a
slice of `[i16; 2]`, where each element is `[left, right]`.

The simplest fixed-rate path is 48,000 Hz at 60 FPS:

```rust,ignore
const FPS_HZ: u32 = 60;
const SAMPLE_RATE_HZ: u32 = 48_000;

struct MyCore {
    silence: Vec<[i16; 2]>,
}

impl Default for MyCore {
    fn default() -> Self {
        Self {
            silence: silent_stereo_frames_for_video_frame(SAMPLE_RATE_HZ, FPS_HZ),
        }
    }
}
```

That creates 800 stereo frames per video frame. Report the same timing to the
frontend:

```rust,ignore
fn av_info(&self) -> SystemAvInfo {
    fixed_system_av_info(WIDTH, HEIGHT, FPS_HZ as f64, SAMPLE_RATE_HZ as f64)
}
```

Submit audio with video when possible:

```rust,ignore
let accepted = runtime.video_refresh_frame_with_audio(
    &self.framebuffer,
    WIDTH,
    HEIGHT,
    pitch,
    &self.silence,
);
```

`accepted` is the number of stereo frames the frontend accepted. Simple silent
examples can ignore it. Streaming cores should watch for short acceptance and
adjust buffering or diagnostics.

For non-divisible rates such as 44,100 Hz at 60 FPS, do not use
`silent_stereo_frames_for_video_frame`; it intentionally requires exact integer
division. Use an accumulator that alternates batch sizes over time so the long
term sample count matches the reported `SystemTiming`.

Audio callback events are separate from normal pushed audio. The normal path is
to submit samples during `run`. Frontend-driven audio callback mode should be
registered through `configure_events` only when the core is designed for that
scheduling model:

```rust,ignore
fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
    events.handle_audio_buffer_status(Self::handle_audio_buffer_status);
}
```

Reference: [Runtime Video and Audio](api/runtime-video-audio.md).
