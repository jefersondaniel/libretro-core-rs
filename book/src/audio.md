# Audio

Libretro audio is pushed by the core while the frontend is running it. During
each `run` call, most cores produce one video frame and the matching amount of
audio for the timing reported by `av_info`.

Samples are signed 16-bit stereo frames. In this crate, a batch is a slice of
`[i16; 2]`, where each element is `[left, right]`. A value of `0` is silence.
Positive and negative values move the waveform around that center point.

The beginner path is:

1. Pick video and audio timing in `av_info`.
2. Keep an audio buffer in the core struct.
3. Fill that buffer during `run`.
4. Submit the video frame and audio batch together.

## Silence First

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

This is useful for early cores because it keeps the frontend audio device fed
while you are still building rendering and input.

## Submit Audio

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

If the core needs separate accounting, submit video first and then use
`runtime.audio_sample_batch(&audio_frames)`. The combined helpers are easier to
read for the common case because they make it obvious that every rendered frame
also submits audio.

## Generate Samples

Generated audio usually belongs in core state. Store phase, oscillators,
resamplers, or decoded audio queues in your core struct, then write the next
batch during `run`:

```rust,ignore
fn fill_square_wave(&mut self) {
    self.audio.clear();

    for _ in 0..self.samples_per_frame {
        let sample = if self.phase < 0.5 { 8_000 } else { -8_000 };
        self.audio.push([sample, sample]);

        self.phase += 440.0 / SAMPLE_RATE_HZ as f32;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }
}
```

Real emulators normally produce audio from the emulated audio hardware rather
than a simple waveform, but the shape is the same: keep audio state in the core,
fill a `[i16; 2]` batch, and submit it during `run`.

## Pacing

For exact fixed timing such as 48,000 Hz at 60 FPS, every video frame gets the
same number of audio frames. For timing where `sample_rate / fps` is not an
integer, such as 48,000 Hz at 59.94 FPS, do not use
`silent_stereo_frames_for_video_frame`; it intentionally requires exact integer
division.

Use an accumulator that alternates batch sizes over time so the long-term sample
count matches the reported `SystemTiming`:

```rust,ignore
fn audio_frames_for_next_video_frame(&mut self) -> usize {
    self.audio_remainder += SAMPLE_RATE_HZ as f64;
    let frames = (self.audio_remainder / VIDEO_FPS).floor() as usize;
    self.audio_remainder -= frames as f64 * VIDEO_FPS;
    frames
}
```

The exact helper shape depends on how the core represents timing, but the rule
is always the same: do not slowly drift away from the `SystemAvInfo` values you
reported to the frontend.

## Frame Time Callback

Frame time is a frontend notification that reports elapsed microseconds between
frames. It is useful when a core can use frontend pacing data instead of
assuming every `run` call is exactly one nominal video frame.

Register it with a single callback:

```rust,ignore
fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
    events.set_frame_time_callback(FrameTime::from_micros(16_667), Self::frame_time);
}

fn frame_time(&mut self, elapsed: FrameTime) {
    self.last_frame_time = elapsed;
}
```

The `reference` value passed to `set_frame_time_callback` is the core's expected
frame interval. For a 60 FPS core, `16_667` microseconds is the usual reference.
The frontend stores one frame-time callback table, so this API uses set/clear
wording instead of add/remove listener wording. Calling
`set_frame_time_callback` again replaces the callback and reference.

## Audio Callbacks

Audio callback events are separate from normal pushed audio. The normal path is
to submit samples during `run`. Frontend-driven audio callback mode should be
registered through `configure_events` only when the core is designed for that
scheduling model:

```rust,ignore
fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
    events.add_audio_buffer_status_listener(Self::audio_buffer_status);
}
```

Reference: [Runtime Video and Audio](api/runtime-video-audio.md).
