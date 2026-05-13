# Content, AV, and Timing

Content loading has two surfaces:

- `ContentContract` tells the frontend which extensions and no-game behavior
  the core supports.
- `GameInfo` gives `load_game` a borrowed view of the frontend-provided path,
  bytes, and metadata.

For no-game or optional-content cores, register the same contract in both
`system_info` and `on_set_environment`:

```rust,ignore
fn content_contract() -> ContentContract {
    ContentContract::new("bin|dat").with_support_no_game(true)
}
```

AV timing is expressed through `SystemAvInfo`, `GameGeometry`, and
`SystemTiming`. Helpers such as `fixed_system_av_info` and
`silent_stereo_frames_for_video_frame` keep common 60 Hz audio pacing explicit
and testable.

Dynamic-rate-control behavior is still driven by the frontend, but the core
must report coherent FPS and sample-rate values. When a core changes geometry or
timing at runtime, use the typed environment methods for system AV updates so
the frontend can resynchronize.

Relevant local references:

- [Developing Libretro Cores](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/developing-cores.md)
- [Dynamic Rate Control](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/dynamic-rate-control.md)
- [Software example](../examples/software-core.md)
