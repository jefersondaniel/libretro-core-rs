# Frontend Services

Frontend services are optional capabilities a core discovers at runtime.
Each one is a typed handle acquired from `Environment` (or `Runtime`); if
the frontend does not expose the service, the accessor returns `None` and
the core should degrade gracefully rather than fail loudly.

```rust,ignore
let Some(rumble) = runtime.environment().rumble_interface() else {
    return; // frontend has no rumble; skip haptics this frame
};
rumble.set_state(InputPort::from(0), RumbleEffect::Strong, RumbleStrength::max());
```

This chapter groups services by what they do. Event-shaped callbacks
(keyboard, audio callback, camera frames, location lifecycle, frame time)
are registered through `CoreEventConfig` instead — see
[Input and Events](input-events.md#event-callback-reference).

## Logging and Messages

`Logger` is the universal output for debug/info/warn/error lines. Acquire
it from `Environment` or `Runtime`; it has no `is_available` because the
frontend always exposes a log target (defaulting to stderr if needed).

```rust,ignore
runtime.logger().info("starting frame");
runtime.logger().warn(format!("fallback path: {reason}"));
```

For user-visible overlays, two helpers exist on `Runtime` and
`Environment`:

```rust,ignore
let _ = runtime.set_message("Save complete", 120); // text, frames

let mut env = runtime.environment();
let _ = env.set_message_ext(
    ExtendedMessage::new("Compiling shaders…")
        .with_duration_millis(2_000)
        .with_target(MessageTarget::Osd)
        .with_kind(MessageKind::Progress)
        .with_progress(MessageProgress::Percent(50)),
);
```

See [Environment and Core Options](environment-options.md#messages-and-logging)
for the full `ExtendedMessage` shape.

## Rumble

```rust,ignore
if let Some(rumble) = env.rumble_interface() {
    rumble.set_state(InputPort::from(0), RumbleEffect::Strong, RumbleStrength::max());
    rumble.set_state(InputPort::from(0), RumbleEffect::Weak, RumbleStrength::new(0x4000));
}
```

`RumbleEffect` is `Strong` or `Weak`. `RumbleStrength` is a `u16` newtype
with `new`, `off`, and `max` constructors. The port is the same player slot
used by joypad polling — see [Input](../input.md).

## LED

```rust,ignore
if let Some(led) = env.led_interface() {
    let _ = led.set_state(LedIndex::new(0), LedState::On);
}
```

`LedIndex::new(i32)` selects which LED to toggle and `LedState` is `On` or
`Off`. The mapping is frontend-specific.

## Sensors

Per-port accelerometer, gyroscope, and illuminance sampling. Enable each
sensor explicitly at the rate you want, poll the typed inputs each frame,
disable when done.

```rust,ignore
if let Some(sensors) = env.sensor_interface() {
    let port = InputPort::from(0);
    let _ = sensors.enable(port, Sensor::Accelerometer, SensorRateHz::new(60));

    let ax = sensors.input(port, SensorInput::AccelerometerX); // Option<f32>
    let ay = sensors.input(port, SensorInput::AccelerometerY);
    let az = sensors.input(port, SensorInput::AccelerometerZ);

    let _ = sensors.disable(port, Sensor::Accelerometer);
}
```

`Sensor` selects which physical sensor to drive. `SensorInput` selects
which axis to read. Reads return `Option<f32>`; `None` means the sensor is
not enabled or not available.

## Microphone

Microphone access is RAII-wrapped. Open the interface, open a stream, read
samples while enabled, drop the handle when done.

```rust,ignore
let Some(interface) = env.microphone_interface() else { return; };
let Some(mut mic) = interface.open_default() else { return; };

mic.set_enabled(true);
let mut buffer = [0i16; 512];
match mic.read_samples(&mut buffer) {
    Ok(count) => self.consume_audio(&buffer[..count]),
    Err(MicrophoneReadError::Unavailable) => self.disable_microphone(),
    Err(_) => {}
}
```

Use `interface.open(MicrophoneParams::new(rate))` for a specific sample
rate. The `Microphone` handle is `!Send`; keep it on the core struct only
while you actually use it.

## MIDI

Byte-level MIDI in/out with microsecond delta tracking:

```rust,ignore
if let Some(midi) = env.midi_interface() {
    if midi.input_enabled() {
        while let Some(byte) = midi.read_byte() {
            self.midi_buffer.push(byte);
        }
    }
    if midi.output_enabled() {
        let _ = midi.write_byte(0x90, MidiDeltaMicros::from(0)); // note on, no delay
        let _ = midi.flush();
    }
}
```

`midi.is_available()` is a const check and useful for early-out before
calling other methods.

## Location

GPS-style position polling.

```rust,ignore
if let Some(location) = env.location_interface() {
    let _ = location.set_interval(
        LocationIntervalMillis::from(1_000),
        LocationIntervalMeters::from(5),
    );
    let _ = location.start();

    if let Some(pos) = location.position() {
        // pos.latitude, pos.longitude, pos.horizontal_accuracy, ...
    }

    let _ = location.stop();
}
```

Lifecycle callbacks (`location_initialized`, `location_deinitialized`) are
event-shaped — see [Input and Events](input-events.md#event-callback-reference).

## Camera

Camera control acquires the interface with the desired delivery mode (raw
framebuffer or OpenGL texture):

```rust,ignore
let Some(camera) = env.camera_interface(CameraRequest::raw_framebuffer()) else {
    return;
};

if camera.capabilities().contains(CameraCapability::RawFramebuffer) {
    let _ = camera.start();
    // Frame delivery arrives through the camera_raw_frame event listener.
}
```

`CameraRequest::open_gl_texture()` selects texture delivery and pairs with
the `camera_texture_frame` event. See
[Input and Events](input-events.md#event-callback-reference) for the
matching frame-callback shapes.

## Performance Counters

`PerfInterface` exposes wall clock, tick counter, CPU feature flags, and
pinned counter management. See
[Diagnostics and Performance](diagnostics-performance.md) for the full
walkthrough; here is the shape:

```rust,ignore
let perf = runtime.environment().perf_interface();
if let Some(perf) = perf {
    let now_us = perf.time_micros();           // Option<PerfTimeMicros>
    let features = perf.cpu_features();        // Option<CpuFeatures>
    if features.map(|f| f.contains(CpuFeature::Avx2)).unwrap_or(false) {
        self.use_simd_path();
    }
}
```

## Netplay Packets

When a core wants to participate in netplay, it installs a packet handler
through `Environment` and receives a `NetpacketSession` from the wrapper:

```rust,ignore
fn on_netpacket_receive(&mut self, session: &NetpacketSession, packet: Netpacket<'_>) {
    let from = packet.client_id();
    self.apply_remote_input(from, packet.data());
}

fn run(&mut self, runtime: &mut Runtime<'_>) {
    if let Some(session) = self.netpacket_session.as_ref() {
        session.send(
            NetpacketTarget::Broadcast,
            NetpacketFlags::reliable(),
            &self.local_input_packet(),
        );
        session.flush(NetpacketTarget::Broadcast);
    }
}
```

`NetpacketTarget` is `Client(NetplayClientId)` or `Broadcast`.
`NetpacketFlags` is built with `reliable()`, `unreliable()`, and
`unsequenced()` constructors that can be combined.

## Frontend State Queries

These are not "services" in the discovery sense, but they are how cores
adapt to frontend mode. All return `Option<...>`:

```rust,ignore
let mut env = runtime.environment();

let lang = env.language();           // localized strings
let user = env.username();           // player profile
let fast = env.fastforwarding();     // skip non-essential work
let throttle = env.throttle_state(); // explicit target rate
let power = env.device_power();      // battery-aware policies
let av = env.audio_video_enable();   // selective frame skip
```

Use them to choose conservative paths when the frontend is fast-forwarding
or running on low battery, and to load locale-aware UI strings.

## Tutorial Cross-Links

- [Input](../input.md), [Audio](../audio.md), [OpenGL](../opengl.md) for
  the corresponding subsystem tutorials.
- [Input and Events](input-events.md#event-callback-reference) for
  event-shaped notifications.
- [Compatibility OpenGL example](../examples/retrocompat-core.md) is the
  reference for performance counters + diagnostics in a real core.
