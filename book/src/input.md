# Input

Libretro input is frontend-mapped. A core asks for abstract devices such as
RetroPad, analog sticks, mouse, pointer, keyboard, and lightgun; the frontend
maps physical hardware to those abstractions.

Most gameplay input is polled during `run`. Call `runtime.poll_input()` once per
frame before reading state:

```rust,ignore
fn run(&mut self, runtime: &mut Runtime<'_>) {
    runtime.poll_input();

    if runtime.joypad_pressed(0, JoypadButton::A) {
        self.jump();
    }

    let x = runtime.analog_axis(0, AnalogStick::Left, AnalogAxis::X);
}
```

Ports are player/controller slots. Port `0` is the first player.

## Polled Devices

Use RetroPad first when it can express the controls. It gives frontends the
widest mapping freedom.

- Joypad: `joypad_pressed` for individual buttons, `joypad_buttons` for a
  bitmask when the frontend supports bitmask queries.
- Analog: `analog_axis` returns signed libretro axis values; `analog_button`
  returns analog button pressure.
- Mouse: `mouse_axis` returns relative movement since the last poll.
- Pointer: `pointer_axis`, `pointer_pressed`, `pointer_count`, and
  `pointer_is_offscreen` represent absolute touch or pen-like input.
- Lightgun: use `LightgunAxis::ScreenX`, `LightgunAxis::ScreenY`,
  `LightgunButton::Trigger`, `LightgunButton::Reload`, and
  `lightgun_is_offscreen`.

Do not normalize raw input ranges in shared helpers unless the core owns that
policy. Keeping libretro-space values visible makes calibration and frontend
quirks easier to diagnose.

## Descriptors And Controller Info

Input descriptors label controls for frontend UIs:

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    let _ = env.set_input_descriptors(&[
        InputDescriptor::joypad(0, JoypadButton::A, "Jump"),
        InputDescriptor::analog(0, AnalogStick::Left, AnalogAxis::X, "Move"),
    ]);
}
```

Controller info declares which controller abstractions a port can use. It is
separate from runtime polling: declare selectable devices through
`set_controller_info`, then poll the matching base abstraction in `run`.

Override `set_controller_port_device` when the core needs to react to frontend
controller selection.

## Keyboard Events

Keyboard input is event-shaped in this Rust API. Register the handler next to
the handler method:

```rust,ignore
impl Core for MyCore {
    fn configure_events(&mut self, events: &mut CoreEventConfig<Self>) {
        events.handle_keyboard_event(Self::handle_keyboard_event);
    }
}

impl MyCore {
    fn handle_keyboard_event(&mut self, event: KeyboardEvent) {
        if event.down {
            let key = event.key;
            let text = event.character.as_char();
        }
    }
}
```

Use `KeyboardCharacter` for layout-aware text input and `KeyboardKey` for
semantic special keys.

Reference: [Input and Events](api/input-events.md).
