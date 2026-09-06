# AI Agent Guidance

## Project Scope

This repository is a generic Rust workspace for building libretro cores and
OpenGL-backed libretro helpers. Keep examples, tests, comments, package
metadata, and fixture strings reusable and product-neutral.

## API Design Principles

Developer ergonomics is the primary design constraint for this workspace. The
libretro API should let core authors express frontend intent without manual FFI
ceremony. OpenGL rendering uses standard glow, including its unsafe calls and
named constants; the library owns frontend negotiation and procedure loading.

Follow these principles when adding or changing APIs:

- Keep public APIs Rust-first even when the underlying ABI is C-first.
- Libretro callback/service APIs remain Rust-first and hide ABI unsafety.
- Keep libretro arguments typed: use narrow enums/newtypes instead of raw numeric
  flags, and prefer Rust strings, slices, Option, Result, and owned return values.
- Keep raw pointers, mutable out-parameters, CString/CStr conversion, and ABI
  helper functions behind explicit internal boundaries.
- OpenGL commands deliberately use re-exported `glow`, including its unsafe
  methods and standard constants. Do not rebuild a custom typed GL dispatcher.
- Explain current-context, resource ownership, and buffer safety at each example's
  unsafe boundary. The frontend owns the GL context; glow only loads commands.
- Keep `Runtime::create_glow_context` convenient and restrict initialization to
  the hardware context-reset callback. No GL calls belong in arbitrary Drop paths.
- Preserve upstream contracts exactly, including multi-part callback results,
  pointer lifetime rules, hardware-render lifecycle rules, and context-family
  capability differences.
- Add higher-level helpers only when they remove real repeated setup that most
  cores would otherwise need to copy.
- Keep examples complete and ergonomic. Even small examples should show the
  full practical lifecycle: content contract, AV timing, input polling, audio
  pacing, frame submission, context reset, context destroy, and cleanup.
- Prefer visible, diagnosable failure paths over silent black screens or
  low-information errors.

For APIs that naturally represent combinable flags, consider `enumflags2`
before introducing ad hoc integer bitmasks or loosely typed flag wrappers. Use a
bitflag set only when combinations are valid for the target operation; otherwise
keep separate enums or typed helper methods.

## Libretro Callback Design Choices

Avoid callback APIs that rely on temporal coupling or split intent across
separate declarations. A core author must not have to remember to call a setup
method before an implemented listener can ever run. A method named
`keyboard_event` is fine when it is passed directly to
`events.add_keyboard_event_listener(Self::keyboard_event)`, but it must not be
discovered implicitly or depend on an unrelated manual
`env.set_keyboard_callback()` call in normal ergonomic workflows. Likewise, do
not replace that with a separate opt-in flag such as
`event_subscriptions().with_keyboard_events()` plus a detached
`keyboard_event()` method; that still lets the listener and registration drift
apart.

For libretro callbacks, prefer designs that make registration declarative and
colocated with the callback:

- Event-shaped callbacks should use an internal event bus or typed
  configuration layer, not ad hoc public trampoline setup. A good high-level
  shape is `events.add_keyboard_event_listener(Self::keyboard_event)`: a single
  DOM-style listener registration method that records both the opt-in and the
  callback.
- Use DOM-style `add_*_listener` and `remove_*_listener` names in public
  listener registration methods. Multiple listeners for the same event should
  dispatch in registration order, and removing a listener should use the same
  callback function pointer that was added.
- Prefer names such as `add_keyboard_event_listener`,
  `remove_keyboard_event_listener`, `add_audio_buffer_status_listener`, and
  `add_camera_raw_frame_listener` over handler-shaped names such as `handle_*`,
  prepositional names such as `on_keyboard_event`, or low-level names such as
  `set_keyboard_callback`.
- Do not force single-registration callback hooks into listener naming.
  Frame-time notifications carry one frontend reference interval and one active
  callback, so use names such as `set_frame_time_callback` and
  `clear_frame_time_callback` for that surface.
- Listener callback methods may be named after the event, such as
  `keyboard_event`, `frame_time`, or `audio_buffer_status`, when they are
  registered directly beside the callback in `configure_events`.
- Listener callbacks should receive typed Rust values and, when they need core
  state, dispatch as `fn(&mut CoreType, Event)` or equivalent. Avoid requiring
  `'static` closures that force users into shared mutable containers just to
  access their core state.
- Registering an event listener must automatically perform the matching
  `RETRO_ENVIRONMENT_SET_*_CALLBACK` negotiation at the correct lifecycle
  point. Core authors should not need to remember libretro callback ordering.
- Keep raw C callback tables, global trampolines, and pointer conversions behind
  private/internal boundaries.

Do not force every libretro callback into the same abstraction. Separate the
surface by semantics:

- Events are notifications that can be forwarded through an event bus, such as
  keyboard events, audio buffer status, camera frames, and location lifecycle
  notifications.
- Callback-shaped hooks have one active registration or one negotiated value,
  such as frame-time notifications with their reference interval.
- Services are frontend-owned interfaces with methods, such as rumble, LED,
  VFS, MIDI, microphone, and performance counters.
- Queries and commands return a specific answer to the frontend, such as disk
  control, proc-address lookup, netplay accept/reject, and core-options display
  updates. These should use typed methods or traits that make the single result
  explicit rather than a multicast event.

When preserving existing lower-level `set_*_callback` names for compatibility,
keep them out of examples and normal documentation. New ergonomic APIs should
register listeners directly, for example
`events.add_keyboard_event_listener(...)`, rather than adding separate
`enable_*` methods that can be forgotten or drift away from the listener. This
does not apply to callback-shaped hooks whose ergonomic public API is already a
typed `set_*_callback` method, such as `set_frame_time_callback`.

## Commit Messages

Always use Conventional Commits for commit messages.

Allowed examples:

- `feat: add content override helper`
- `fix: preserve system info pointer lifetime`
- `test: cover GLES2 texture cleanup`
- `docs: document workspace layout`
- `refactor: split diagnostic text setup`
- `chore: update package metadata`

Use a scope when it adds clarity, for example:

- `fix(glow): gate extension aliases on live context support`
- `test(wrapper): reject undersized software frames`

Avoid vague messages such as `update`, `changes`, or `fix stuff`.

## Validation

Before finishing changes, run:

```sh
cargo test --workspace
```
