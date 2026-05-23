# Environment and Core Options

`Environment` is the typed wrapper around libretro's `RETRO_ENVIRONMENT_*`
commands. A core uses it for two distinct jobs:

- **Setup negotiation** during `on_set_environment` and `load_game`:
  declare content support, pixel format, input descriptors, controller info,
  core options, and hardware rendering.
- **Runtime queries and updates** through `runtime.environment()`: read
  language, paths, throttle state, current options, and push messages or
  geometry changes.

Most environment calls return `bool` (or `Option<T>`) so a core can tell
unsupported vs rejected vs successful. Treat a `false` return as "the
frontend declined this specific request", not as a panic condition.

## Setup Negotiation

`on_set_environment` is the canonical setup hook. The frontend calls it
once, before content load, when the environment callback is first wired up.

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    let _ = ContentContract::new("bin|dat")
        .with_support_no_game(true)
        .register_environment(env);

    let _ = env.set_input_descriptors(&[
        InputDescriptor::joypad(0, JoypadButton::A, "Jump"),
        InputDescriptor::joypad(0, JoypadButton::B, "Fire"),
    ]);

    let _ = env.set_controller_info(&[
        ControllerInfo::new(vec![
            ControllerDescription::new("Gamepad", ControllerDevice::Joypad),
        ]),
    ]);
}
```

A few more setup-time commands you usually want:

```rust,ignore
let _ = env.set_pixel_format(PixelFormat::Xrgb8888);
let _ = env.set_performance_level(PerformanceLevel::new(2));
let _ = env.set_support_no_game(true);
```

For hardware rendering, see [Hardware Rendering and OpenGL](hardware-opengl.md).
For content, see [Content, AV, and Timing](content-av-timing.md).

## Core Options

Two installation paths exist because libretro evolved the options API. The
crate exposes both and lets the core install whichever version the frontend
supports.

The modern path is `set_core_options_v2`, which accepts a `CoreOptions`
value built from typed definitions and optional categories:

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    if env.core_options_version().supports_v2() {
        let options = CoreOptions::new([
            CoreOptionDefinition::new("my_core_mode", "Mode", "auto")
                .with_values([
                    CoreOptionValue::new("auto").with_label("Auto-detect"),
                    CoreOptionValue::new("fast").with_label("Fast"),
                    CoreOptionValue::new("accurate").with_label("Accurate"),
                ])
                .with_info("Picks between speed and accuracy.")
                .with_category("performance"),
        ])
        .with_categories([
            CoreOptionCategory::new("performance", "Performance"),
        ]);

        let _ = env.set_core_options_v2(options);
    } else {
        let _ = env.set_variables(&[
            VariableDefinition::new(
                "my_core_mode",
                "Mode; auto|fast|accurate",
            ),
        ]);
    }
}
```

`core_options_version()` returns a value with `supports_v1()` and
`supports_v2()` flags. Use the v2 path on modern frontends and fall back to
`set_variables` (legacy `VariableDefinition`) only when needed — the v2
storage retains all strings so the frontend can read descriptors safely.

When option visibility should change at runtime, use
`set_core_option_display(CoreOptionDisplay::new("my_core_mode", false))` to
hide an option and `set_variable(key, value)` to push a value back to the
frontend.

## Reading Option Values

During `run`, read current values from the frontend through
`Environment::get_variable`:

```rust,ignore
fn run(&mut self, runtime: &mut Runtime<'_>) {
    let mut env = runtime.environment();
    if env.variables_updated() {
        if let Some(mode) = env.get_variable("my_core_mode") {
            self.apply_mode(&mode);
        }
    }
    // ... advance frame
}
```

`variables_updated()` returns `true` once after the frontend changes any
option, so you can avoid re-applying the same value every frame.

## Messages and Logging

Two message surfaces exist. The simple one shows a string for a number of
frames:

```rust,ignore
let _ = runtime.set_message("Loading failed", 180);
runtime.logger().error("renderer init failed");
```

The richer one uses `ExtendedMessage` with target, kind, level, and
progress:

```rust,ignore
let mut env = runtime.environment();
if env.message_interface_version().is_some() {
    let msg = ExtendedMessage::new("Compiling shaders…")
        .with_duration_millis(2_000)
        .with_target(MessageTarget::Osd)
        .with_kind(MessageKind::Progress)
        .with_progress(MessageProgress::Percent(50));
    let _ = env.set_message_ext(msg);
}
```

Variants worth knowing:

- `MessageTarget::All`, `Osd`, `Log` — where the message goes.
- `MessageKind::Notification`, `NotificationAlt`, `Status`, `Progress`.
- `MessageProgress::Indeterminate` or `MessageProgress::Percent(0..=100)`.

`Logger` is acquired from either `env.logger()` or `runtime.logger()` and
exposes `debug`, `info`, `warn`, `error`.

## Useful Runtime Queries

These are safe to call from `run` and return `None` (or `false`) when the
frontend has not negotiated the capability:

```rust,ignore
let lang = env.language();                  // Option<Language>
let user = env.username();                  // Option<String>
let fast = env.fastforwarding();            // Option<bool>
let throttle = env.throttle_state();        // Option<ThrottleState>
let av = env.audio_video_enable();          // Option<AvEnableFlags>
let dirs = env.save_directory();            // Option<String>
let jit = env.jit_capable();                // Option<bool>
let dupe = env.can_dupe_frames();           // Option<bool>
let dev = env.input_device_capabilities();  // Option<InputDeviceCapabilities>
```

Many of them are also useful at setup: query path directories early to
preallocate buffers, query input device capabilities to refine controller
info.

## Service Discovery

Optional services are acquired by calling typed `*_interface` accessors on
`Environment`. Each returns `Option<...>`; treat `None` as "not exposed by
this frontend" and degrade gracefully.

```rust,ignore
let rumble = env.rumble_interface();
let led = env.led_interface();
let sensors = env.sensor_interface();
let perf = env.perf_interface();
let mic = env.microphone_interface();
let midi = env.midi_interface();
let location = env.location_interface();
let camera = env.camera_interface(CameraRequest::raw_framebuffer());
```

For details on each interface, see
[Frontend Services](frontend-services.md).

## Setup vs Runtime

| Phase | Typical commands |
| --- | --- |
| `on_set_environment` | `set_*` (pixel format, descriptors, controller info, variables/options, performance level, support flags, subsystems, memory maps, disk control, AV info hints) |
| `load_game` | `set_pixel_format`, `set_hw_render_from_candidates`, `set_serialization_quirks` |
| `run` (via `runtime.environment()`) | `get_variable`, `variables_updated`, `set_message[_ext]`, `set_geometry`, `set_system_av_info`, throttle/fastforward queries, service acquisition |
| Any | logger, language, paths, message version, service `is_available` checks |

Some commands work in both phases but mean different things. Setting a
controller descriptor during `on_set_environment` declares the labels;
calling it from `run` typically only succeeds if the frontend is willing to
accept hot updates. When in doubt, check the return value.

## Reference

- [Core Options Translation](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/core-options-translation.md) — upstream
  source of truth for v0/v1/v2 differences.
- [Frontend Services](frontend-services.md) — service-by-service detail.
- [Input and Events](input-events.md) — `CoreEventConfig` is the event
  surface for callbacks (keyboard, audio, camera, location, frame time).
