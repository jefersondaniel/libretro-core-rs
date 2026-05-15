# Frame Loop Basics

`Core::run` is called once for each frontend frame. Keep it predictable:

1. Poll input.
2. Advance core state.
3. Produce video.
4. Produce audio.
5. Submit the frame.

Software cores usually submit one framebuffer and one audio batch together:

```rust,ignore
fn run(&mut self, runtime: &mut Runtime<'_>) {
    runtime.poll_input();

    self.framebuffer.fill(BLUE_0RGB1555);
    let pitch = WIDTH as usize * core::mem::size_of::<u16>();
    let accepted = runtime.video_refresh_frame_with_audio(
        &self.framebuffer,
        WIDTH,
        HEIGHT,
        pitch,
        &self.silence,
    );

    if accepted != self.silence.len() {
        let _ = runtime.set_message("audio batch was partially accepted", 120);
    }
}
```

`pitch` is the number of bytes between the start of adjacent rows. For a tightly
packed `u16` framebuffer, it is `width * size_of::<u16>()`.

The combined helpers submit video first and audio second. Their return value is
the number of stereo audio frames accepted by the frontend. Minimal examples can
ignore that value; streaming audio cores should track short writes.

For software frame validation and detailed error handling, split the calls:

```rust,ignore
if runtime.video_refresh_frame(&self.framebuffer, WIDTH, HEIGHT, pitch) {
    let _ = runtime.audio_sample_batch(&self.silence);
} else {
    let _ = runtime.video_refresh_dupe_with_audio(WIDTH, HEIGHT, &self.silence);
}
```

After hardware rendering has been accepted, fallback paths should submit a
hardware frame or duplicate frame with audio. Do not switch to software pixels
inside an active hardware-render path.
