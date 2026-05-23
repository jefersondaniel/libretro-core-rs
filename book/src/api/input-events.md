# Input and Events

Libretro has two input shapes, and the Rust API keeps them distinct.

Polled input is requested by the core during `run`. Call `runtime.poll_input()`
once per frame, then use typed helpers:

```rust,ignore
runtime.poll_input();

if runtime.joypad_pressed(0, JoypadButton::A) {
    // advance game state
}

let x = runtime.analog_axis(0, AnalogStick::Left, AnalogAxis::X);
```

Ports are player/controller slots. Port `0` is the first player. Prefer
`joypad_pressed` for beginner snippets; use `joypad_buttons` when scanning
several RetroPad buttons from the same port.

## Gamepad Support

RetroPad is the default gamepad abstraction. Frontends map physical controllers
to `JoypadButton` values, so core code can stay portable:

```rust,ignore
if runtime.joypad_pressed(0, JoypadButton::B) {
    self.fire();
}

let buttons = runtime.joypad_buttons(0);
let mut horizontal = 0;
if buttons.contains(JoypadButton::Right) {
    horizontal += 1;
}
if buttons.contains(JoypadButton::Left) {
    horizontal -= 1;
}
```

Use `Environment::input_device_capabilities` during environment negotiation when
the core wants to tailor setup to devices the frontend reports. Keep the normal
runtime path typed and simple: poll once, then read joypad, analog, mouse,
pointer, or lightgun state.

Mouse axes are relative deltas. Pointer and modern lightgun axes are absolute
screen-space values. Analog, pointer, and lightgun helpers return libretro-space
raw values, so normalize them only when the core owns the policy.

Use `Environment::set_input_descriptors` to label controls and
`Environment::set_controller_info` to advertise selectable controller types.
Override `set_controller_port_device` when a core needs to react to frontend
controller selection.

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    let _ = env.set_input_descriptors(&[
        InputDescriptor::joypad(0, JoypadButton::B, "Fire"),
        InputDescriptor::joypad(0, JoypadButton::A, "Jump"),
    ]);

    let _ = env.set_controller_info(&[
        ControllerInfo::new(vec![
            ControllerDescription::new("Gamepad", ControllerDevice::Joypad),
        ]),
    ]);
}
```

Descriptors are labels. Controller info advertises selectable controller
abstractions. The actual per-frame state still comes from `Runtime` polling.

## Event Callbacks

Event callbacks are frontend-to-core notifications. Register them in
`configure_events` using DOM-style listener methods. The library then installs
the raw frontend callback during environment setup; normal core code does not
call a separate raw callback setup method.

```rust,ignore
impl Core for MyCore {
    fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
        events.add_keyboard_event_listener(Self::keyboard_event);
    }
}

impl MyCore {
    fn keyboard_event(&mut self, event: KeyboardEvent) {
        if event.down {
            let text = event.character.as_char();
            let key = event.key;
        }
    }
}
```

The listener API follows DOM-style add/remove semantics. You can add multiple
listeners for the same event, duplicate callback registrations are ignored, and
listeners run in registration order. Call the matching `remove_*_listener`
method with the same callback function to remove a listener during
configuration. Callback-shaped hooks that have one active frontend registration
keep set/clear wording; for example, frame timing uses
`set_frame_time_callback(reference, callback)` and `clear_frame_time_callback()`.

```rust,ignore
fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
    events
        .add_keyboard_event_listener(Self::keyboard_event)
        .remove_keyboard_event_listener(Self::keyboard_event);
}
```

Use `KeyboardCharacter` for layout-aware text input. Use `KeyboardKey` for
semantic special keys, and provide configurable bindings when physical keyboard
layout matters.

Other event-shaped surfaces include audio callbacks, audio buffer status,
location lifecycle, and camera lifecycle/frame notifications. Frame timing is a
single callback-shaped hook because the frontend receives one reference
interval. Joypad, analog, mouse, pointer, and lightgun input remain polled
because libretro exposes them that way.

## Event Callback Reference

Register event callbacks from `Core::configure_events`. Every callback receives
`&mut self` as its first argument through the method pointer you pass to
`CoreEventConfig`; the table below shows the additional event payload, if any.

| Frontend notification | Register | Remove or clear | Callback method shape |
| --- | --- | --- | --- |
| Keyboard key/text event | `add_keyboard_event_listener(Self::keyboard_event)` | `remove_keyboard_event_listener(Self::keyboard_event)` | `fn keyboard_event(&mut self, event: KeyboardEvent)` |
| Audio callback request | `add_audio_callback_listener(Self::audio_callback)` | `remove_audio_callback_listener(Self::audio_callback)` | `fn audio_callback(&mut self)` |
| Audio callback active state changed | `add_audio_callback_state_changed_listener(Self::audio_callback_state_changed)` | `remove_audio_callback_state_changed_listener(Self::audio_callback_state_changed)` | `fn audio_callback_state_changed(&mut self, state: AudioCallbackState)` |
| Audio buffer status changed | `add_audio_buffer_status_listener(Self::audio_buffer_status)` | `remove_audio_buffer_status_listener(Self::audio_buffer_status)` | `fn audio_buffer_status(&mut self, status: AudioBufferStatus)` |
| Frame time reported | `set_frame_time_callback(reference, Self::frame_time)` | `clear_frame_time_callback()` | `fn frame_time(&mut self, elapsed: FrameTime)` |
| Location initialized | `add_location_initialized_listener(Self::location_initialized)` | `remove_location_initialized_listener(Self::location_initialized)` | `fn location_initialized(&mut self)` |
| Location deinitialized | `add_location_deinitialized_listener(Self::location_deinitialized)` | `remove_location_deinitialized_listener(Self::location_deinitialized)` | `fn location_deinitialized(&mut self)` |
| Camera initialized | `add_camera_initialized_listener(Self::camera_initialized)` | `remove_camera_initialized_listener(Self::camera_initialized)` | `fn camera_initialized(&mut self)` |
| Camera deinitialized | `add_camera_deinitialized_listener(Self::camera_deinitialized)` | `remove_camera_deinitialized_listener(Self::camera_deinitialized)` | `fn camera_deinitialized(&mut self)` |
| Camera raw frame | `add_camera_raw_frame_listener(Self::camera_raw_frame)` | `remove_camera_raw_frame_listener(Self::camera_raw_frame)` | `fn camera_raw_frame(&mut self, frame: CameraRawFrame<'_>)` |
| Camera texture frame | `add_camera_texture_frame_listener(Self::camera_texture_frame)` | `remove_camera_texture_frame_listener(Self::camera_texture_frame)` | `fn camera_texture_frame(&mut self, frame: CameraTextureFrame)` |

Payload types keep libretro units but hide raw callback table details:

- `KeyboardEvent` has `down`, `key`, `character`, and `modifiers`. Use
  `KeyboardCharacter::as_char()` when you need layout-aware text.
- `AudioCallbackState` is `Active` or `Inactive`. The request callback has no
  extra payload; it means the frontend is asking the core to produce audio for
  callback-driven audio mode.
- `AudioBufferStatus` has `active`, `occupancy`, and `underrun_likely`.
  `occupancy.percent()` returns `Some(0..=100)` for normal frontend values;
  `raw_percent()` preserves out-of-range frontend data for diagnostics.
- `FrameTime` stores signed microseconds. Use `FrameTime::as_micros()` when you
  need the numeric elapsed time.
- Location lifecycle callbacks have no extra payload. They tell the core when
  the frontend-owned location interface has initialized or deinitialized.
- Camera lifecycle callbacks have no extra payload. Camera frame callbacks carry
  either `CameraRawFrame<'_>` with `pixels`, `width`, `height`, and
  `pitch_bytes`, or `CameraTextureFrame` with `texture_id`, `texture_target`,
  and a 3x3 `affine` transform.

Tutorial: [Input](../input.md).

Reference: [Libretro Input API](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/input-api.md).
