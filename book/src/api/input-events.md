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
`configure_events` using verb-based handlers. The library then installs the raw
frontend callback during environment setup; normal core code does not call a
separate raw callback setup method.

```rust,ignore
impl Core for MyCore {
    fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
        events.handle_keyboard_event(Self::handle_keyboard_event);
    }
}

impl MyCore {
    fn handle_keyboard_event(&mut self, event: KeyboardEvent) {
        if event.down {
            let text = event.character.as_char();
            let key = event.key;
        }
    }
}
```

Use `KeyboardCharacter` for layout-aware text input. Use `KeyboardKey` for
semantic special keys, and provide configurable bindings when physical keyboard
layout matters.

Other event-shaped surfaces include audio callbacks, audio buffer status,
frame-time notifications, location lifecycle, and camera lifecycle/frame
notifications. Joypad, analog, mouse, pointer, and lightgun input remain polled
because libretro exposes them that way.

Tutorial: [Input](../input.md).

Reference: [Libretro Input API](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/input-api.md).
