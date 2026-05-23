# Content, AV, and Timing

Three things have to agree before a frontend will accept frames from a core:
the **content contract** (which file extensions and no-game modes are
supported), the **load result** (whether `load_game` accepted the supplied
content), and the **AV info** (geometry, FPS, and sample rate). This chapter
covers those three surfaces and the helpers that keep them consistent.

## Content Contract

`ContentContract` is a builder that captures everything the frontend needs to
know about content support up front:

```rust,ignore
fn content_contract() -> ContentContract {
    ContentContract::new("bin|dat")
        .with_support_no_game(true)
        .with_persistent_data(true)
        .with_need_fullpath(false)
        .with_block_extract(false)
}
```

The same builder is used in two places, so the frontend sees one consistent
view. From `system_info`, apply it to the metadata:

```rust,ignore
fn system_info(&self) -> SystemInfo {
    let mut info = SystemInfo::new("my-core", env!("CARGO_PKG_VERSION"));
    content_contract().apply_to_system_info(&mut info);
    info
}
```

From `on_set_environment`, register the runtime side:

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    let _ = content_contract().register_environment(env);
}
```

`apply_to_system_info` writes `valid_extensions`, `need_fullpath`, and
`block_extract`. `register_environment` calls `set_support_no_game` and
installs the content-info overrides. Keep the helper free of side effects so
it can be called from both lifecycle points.

The four flags are independent:

| Flag | Meaning |
| --- | --- |
| `with_support_no_game(true)` | Core can run without content (`load_game(None, ...)` returns `true`). |
| `with_need_fullpath(true)` | Frontend supplies a filesystem path instead of a borrowed byte slice. Useful for emulators that mmap. |
| `with_block_extract(true)` | Frontend should not auto-extract archives before handing content to the core. |
| `with_persistent_data(true)` | Core promises the borrowed `data` slice survives past `load_game`. Required when reusing the slice from `run`. |

## GameInfo

`load_game` receives `Option<GameInfo<'_>>`. `None` means no-game startup
(only valid if you set `with_support_no_game(true)`). `Some(info)` borrows
three optional fields from the frontend:

```rust,ignore
fn load_game(&mut self, game: Option<GameInfo<'_>>, runtime: &mut Runtime<'_>) -> bool {
    let Some(game) = game else {
        return true; // no-game start
    };

    if let Some(path) = game.path_lossy() {
        runtime.logger().info(format!("loading {path}"));
    }

    let bytes = game.data.unwrap_or(&[]);
    self.load_rom(bytes)
}
```

The fields are `path: Option<&CStr>`, `data: Option<&[u8]>`, and
`meta: Option<&CStr>`. Use `path_lossy()` and `meta_lossy()` when you want
`Cow<'_, str>` instead of raw `CStr`. The borrow ends when `load_game`
returns unless `with_persistent_data(true)` is set.

Return `false` from `load_game` to refuse the content. The frontend then
shows the failure to the user instead of running a broken core. Use
`runtime.logger()` or `runtime.set_message(...)` so the failure is
diagnosable.

## SystemAvInfo

`av_info` reports geometry and timing in one value:

```rust,ignore
fn av_info(&self) -> SystemAvInfo {
    fixed_system_av_info(320, 240, 60.0, 48_000.0)
}
```

`SystemAvInfo` decomposes into `GameGeometry` and `SystemTiming`:

| Field | Type | Notes |
| --- | --- | --- |
| `geometry.base_width` / `base_height` | `u32` | The current visible frame size. |
| `geometry.max_width` / `max_height` | `u32` | The largest frame the core will ever submit. Frontend may allocate for this size. |
| `geometry.aspect_ratio` | `f32` | Use `0.0` to let the frontend derive aspect from base size. |
| `timing.fps` | `f64` | Target frame rate. |
| `timing.sample_rate` | `f64` | Audio sample rate in Hz. |

Three builders cover the common shapes:

```rust,ignore
let g = game_geometry(320, 240);                       // base == max
let g = bounded_game_geometry(320, 240, 640, 480);     // distinct base / max
let av = system_av_info(g, 60.0, 48_000.0);
let av = fixed_system_av_info(320, 240, 60.0, 48_000.0); // shorthand for both
```

For audio pacing helpers, see [Audio](../audio.md). The most useful are
`silent_stereo_frames_for_video_frame(sample_rate, fps)` (panics on
non-integer division) and `exact_audio_frames_per_video_frame(sample_rate, fps)`.

## Dynamic Geometry and Timing

If the visible image resizes during gameplay (a console switching between
240p and 480i, or a windowed app changing internal resolution), update the
frontend through `Runtime::set_geometry`:

```rust,ignore
fn run(&mut self, runtime: &mut Runtime<'_>) {
    if self.resolution_changed {
        runtime.set_geometry(game_geometry(self.width, self.height));
        self.resolution_changed = false;
    }
    // ... submit frame
}
```

`set_geometry` only adjusts the geometry block; the frontend keeps its
existing timing. Use `Environment::set_system_av_info` when both geometry
and timing change at runtime — it replaces the entire `SystemAvInfo` and
gives the frontend a chance to resynchronize audio and video.

```rust,ignore
let mut env = runtime.environment();
let _ = env.set_system_av_info(fixed_system_av_info(640, 480, 59.94, 48_000.0));
```

Reporting `max_width`/`max_height` larger than any frame you actually submit
is fine and avoids reallocation when geometry changes inside those bounds.

## Reference

- [Software example](../examples/software-core.md) — minimal contract + fixed AV info.
- [Modern OpenGL example](../examples/demo-core.md) — content optional, HW
  rendering negotiated after `load_game`.
- [Developing Libretro Cores](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/developing-cores.md)
- [Dynamic Rate Control](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/dynamic-rate-control.md)
