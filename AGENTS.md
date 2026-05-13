# AI Agent Guidance

## Project Scope

This repository is a generic Rust workspace for building libretro cores and
OpenGL-backed libretro helpers. Keep examples, tests, comments, package
metadata, and fixture strings reusable and product-neutral.

## API Design Principles

Developer ergonomics is the primary design constraint for this workspace. The
public API should let core authors express libretro and OpenGL intent without
manual FFI ceremony, unsafe blocks, or raw numeric constants at call sites.

Follow these principles when adding or changing APIs:

- Keep public APIs Rust-first even when the underlying ABI is C-first.
- Do not expose `unsafe` in normal core-author workflows. Unsafe code belongs
  behind small, audited wrapper boundaries.
- Do not require callers to pass magic numbers or raw GL/libretro flags when a
  typed enum, newtype, builder, or helper can encode the valid choices.
- Prefer typed enums for OpenGL targets, formats, capabilities, draw modes,
  buffer usages, texture parameters, and libretro context/device choices.
- Use the narrowest applicable enum for an operation so callers cannot pass a
  flag that is meaningless for that context.
- For OpenGL wrappers, keep public method names easy to infer from the OpenGL
  specification. Prefer Rust-style versions of the canonical command names
  (`bind_framebuffer`, `create_shader`, `draw_arrays`) for the safe typed public
  API, even when the argument types are Rust-first wrappers.
- Use explicit implementation-detail names for private/raw helpers. Internal
  ABI-facing helpers may use suffixes such as `_raw`, `_unchecked`, `_ffi`, or
  more specific names when needed to distinguish them from the public typed API.
- Do not make the typed public API harder to discover by adding extra words such
  as `_object`, `_for_program`, or `_checked` when the canonical OpenGL command
  name can safely carry the ergonomic typed signature.
- Prefer Rust-native inputs and outputs: `&str`, slices, enums, `Option`,
  `Result`, owned values, and return values instead of raw pointers, mutable
  out-params, `CString`, or `CStr`.
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
method before an implemented handler can ever run. A method like
`Core::keyboard_event` must not silently depend on an unrelated manual
`env.set_keyboard_callback()` call in normal ergonomic workflows. Likewise, do
not replace that with a separate opt-in flag such as
`event_subscriptions().with_keyboard_events()` plus a detached
`keyboard_event()` method; that still lets the handler and registration drift
apart.

For libretro callbacks, prefer designs that make registration declarative and
colocated with the handler:

- Event-shaped callbacks should use an internal event bus or typed
  configuration layer, not ad hoc public trampoline setup. A good high-level
  shape is `events.handle_keyboard_event(Self::handle_keyboard_event)`: a
  single verb-based method that records both the opt-in and the handler.
- Use verbs in public handler-registration methods. Prefer names such as
  `handle_keyboard_event`, `handle_frame_time`,
  `handle_audio_buffer_status`, and `handle_camera_frame` over noun-only names
  such as `keyboard_events`, prepositional names such as `on_keyboard_event`, or
  low-level names such as `set_keyboard_callback`.
- Event handlers should receive typed Rust values and, when they need core
  state, dispatch as `fn(&mut CoreType, Event)` or equivalent. Avoid requiring
  `'static` closures that force users into shared mutable containers just to
  access their core state.
- Registering an event handler must automatically perform the matching
  `RETRO_ENVIRONMENT_SET_*_CALLBACK` negotiation at the correct lifecycle
  point. Core authors should not need to remember libretro callback ordering.
- Keep raw C callback tables, global trampolines, and pointer conversions behind
  private/internal boundaries.

Do not force every libretro callback into the same abstraction. Separate the
surface by semantics:

- Events are notifications that can be forwarded through an event bus, such as
  keyboard events, frame-time notifications, audio buffer status, camera frames,
  and location lifecycle notifications.
- Services are frontend-owned interfaces with methods, such as rumble, LED,
  VFS, MIDI, microphone, and performance counters.
- Queries and commands return a specific answer to the frontend, such as disk
  control, proc-address lookup, netplay accept/reject, and core-options display
  updates. These should use typed handlers or traits that make the single result
  explicit rather than a multicast event.

When preserving existing lower-level `set_*_callback` names for compatibility,
keep them out of examples and normal documentation. New ergonomic APIs should
register handlers directly, for example `events.handle_keyboard_event(...)`,
rather than adding separate `enable_*` methods that can be forgotten or drift
away from the handler.

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

- `fix(glsym): gate texture arrays on live context version`
- `test(wrapper): reject undersized software frames`

Avoid vague messages such as `update`, `changes`, or `fix stuff`.

## Validation

Before finishing changes, run:

```sh
cargo test --workspace
```
