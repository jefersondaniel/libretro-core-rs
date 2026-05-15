# Runtime Video and Audio

`Runtime` is the per-frame handle passed to `Core::run`. It provides typed
helpers for polling input, submitting software frames, submitting hardware
frames, sending audio, showing frontend messages, and querying services.

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

After hardware rendering has been accepted, submit hardware or duplicate frames
from fallback paths. Do not submit software pixels from an active hardware path.

Tutorials:

- [Frame Loop Basics](../frame-loop-basics.md)
- [Audio](../audio.md)
- [OpenGL](../opengl.md)
