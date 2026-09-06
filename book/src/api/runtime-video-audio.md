# Runtime Video and Audio

`Runtime` is the per-frame handle passed to `Core::run`. It provides typed
helpers for polling input, submitting software frames, submitting hardware
frames, sending audio, showing frontend messages, and querying services.

It is also passed to a few lifecycle methods, such as `load_game` and
`hw_context_reset`, when those methods need frontend access. Do not store
`Runtime` in your core struct. Store your own state, and use the `Runtime`
borrow only during the current callback.

## What Runtime Offers

`Runtime` groups the frontend operations that a running core commonly needs:

| Job | Examples |
| --- | --- |
| Input | `poll_input`, `joypad_pressed`, `joypad_buttons`, `analog_axis`, `mouse_axis`, `pointer_pressed` |
| Video | `video_refresh_frame_with_audio`, `video_refresh_hw_with_audio`, `video_refresh_dupe_with_audio` |
| Audio | `audio_sample`, `audio_sample_batch`, combined video/audio helpers |
| Hardware rendering | `current_framebuffer`, `hw_proc_address` through `Gl::init` |
| Messages and logs | `set_message`, `logger` |
| Environment | `environment()` for runtime-valid environment commands |
| Frontend services | rumble, LED, sensors, camera, microphone, MIDI, VFS, performance, netplay helpers |

`Runtime` is not the core state, a scheduler, or a long-lived service object.
The frontend creates the callback context, the library lends it to your core,
and the borrow ends when the callback returns. Keep emulator state, frame
buffers, audio queues, GL object handles, and user settings on your core struct.
Use `Runtime` to communicate with the frontend during the current callback.

The most common frame shape is:

```rust,ignore
fn run(&mut self, runtime: &mut Runtime<'_>) {
    runtime.poll_input();
    self.update_from_input(runtime);
    self.advance_one_frame();
    self.submit_frame(runtime);
}
```

## Input From Runtime

Calling `poll_input` tells the frontend to refresh its input state for this
frame. After that, typed input helpers read the updated state:

```rust,ignore
runtime.poll_input();

if runtime.joypad_pressed(0, JoypadButton::A) {
    self.jump();
}

let x = runtime.analog_axis(0, AnalogStick::Left, AnalogAxis::X);
```

Ports are player slots. Port `0` is player 1, port `1` is player 2, and so on.
Joypad helpers read the RetroPad abstraction. Analog, mouse, pointer, and
lightgun helpers expose libretro-space raw values so the core can choose its own
normalization and deadzone policy.

For more detail, see [Input](../input.md) and
[Input and Events](input-events.md).

## Video From Runtime

Software cores usually submit one frame and one audio batch together:

```rust,ignore
runtime.poll_input();
let pitch = width as usize * core::mem::size_of::<u16>();
let _ = runtime.video_refresh_frame_with_audio(
    &framebuffer,
    width,
    height,
    pitch,
    &audio_frames,
);
```

The software frame slice, dimensions, and pitch must describe the same image.
Keep the framebuffer in your core state so `run` can update it without
allocating every frame.

## Hardware Rendering From Runtime

Hardware cores render into the frontend-provided framebuffer and then submit a
hardware frame:

```rust,ignore
let Some(framebuffer) = runtime.current_framebuffer() else {
    let _ = runtime.video_refresh_dupe_with_audio(width, height, &audio_frames);
    return;
};

gl.bind_framebuffer(GlFramebufferTarget::Framebuffer, GlFramebuffer::from_raw(framebuffer))?;
runtime.video_refresh_hw_with_audio(width, height, 0, &audio_frames);
```

`Some(0)` identifies the default OpenGL framebuffer and is a valid render target.
`None` means the callback is unavailable or its result cannot represent an OpenGL
framebuffer name.

`current_framebuffer` is valid only while the frontend has an active hardware
context for the current frame. OpenGL function loading is also runtime-backed:
initialize the typed `Gl` facade from the runtime context reset path, then keep
the resulting facade on your core while the context is alive.

For more detail, see [OpenGL](../opengl.md) and
[Hardware Rendering and OpenGL](hardware-opengl.md).

## Audio From Runtime

Prefer the combined video/audio helpers where possible. They make frame pacing
visible in the call site and reduce the chance that a frame returns without
submitting audio.

Helper choices:

| Situation | Helper |
| --- | --- |
| Software pixels plus audio | `video_refresh_frame_with_audio` |
| Hardware-rendered frame plus audio | `video_refresh_hw_with_audio` |
| Duplicate previous frame plus audio | `video_refresh_dupe_with_audio` |
| Separate accounting | `video_refresh_*` plus `audio_sample_batch` |

The `*_with_audio` helpers return the number of stereo audio frames accepted by
the frontend. They are convenient for the common path; split video and audio
calls when the core needs to handle software-frame validation and audio
acceptance independently.

Audio batches are slices of `[i16; 2]`, where each element is one signed 16-bit
stereo frame. Keep generated samples, silence buffers, decoded queues, and
resampling state on your core. Use `Runtime` only to submit the samples.

For more detail, see [Audio](../audio.md).

## Environment From Runtime

Some environment commands are needed after setup. Use `runtime.environment()` to
get a typed `Environment` view:

```rust,ignore
let mut env = runtime.environment();
let _ = env.set_message("Paused", 120);
```

For simple frontend messages, `Runtime` also exposes convenience helpers:

```rust,ignore
let _ = runtime.set_message("Loading failed", 180);
runtime.logger().warn("using fallback renderer");
```

## Frontend Services

Some frontend-owned services are discovered or driven through runtime-valid
environment calls: rumble, LED, sensors, camera, microphone, MIDI, VFS,
performance counters, and netplay helpers. Treat them as optional frontend
capabilities. Query or negotiate them through the typed API, keep any core-side
state in your core struct, and continue to produce diagnosable frames when a
service is unavailable.

For more detail, see [Frontend Services](frontend-services.md).

After hardware rendering has been accepted, submit hardware or duplicate frames
from fallback paths. Do not submit software pixels from an active hardware path.

Tutorials:

- [Frame Loop Basics](../frame-loop-basics.md)
- [Audio](../audio.md)
- [OpenGL](../opengl.md)
