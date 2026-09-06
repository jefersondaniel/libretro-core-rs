# Raw ABI Boundaries

The raw `libretro.h` mapping in this crate exists so the wrapper can
preserve ABI contracts exactly. It is **not** the normal core-author API.
Most cores never need to touch raw types.

The raw module lives at `crates/libretro-core/src/raw.rs` and is
auto-generated from upstream `libretro.h`. Layout assertions live next to
each typed wrapper that crosses the ABI boundary so a future bindgen
update cannot silently break field offsets.

## When To Reach For Raw Types

Three legitimate reasons:

1. **Auditing the wrapper.** Reading the typed API alongside the raw
   bindings is the fastest way to confirm a callback ordering, retained
   string, or nullable function pointer is handled correctly.
2. **Layout tests.** When a new typed feature wraps an upstream struct,
   add `static_assert`-style tests that compare offsets between the typed
   and raw versions. Existing examples live in `crates/libretro-core/tests/`.
3. **Adding a new typed feature.** When `libretro.h` exposes something the
   wrapper does not yet model, the raw types are how you reach it during
   prototyping. The result should land as a typed API; do not leave raw
   types in the public surface.

For everything else, prefer:

- `Core` and `export_core!` instead of writing `retro_*` exports by hand.
- `Environment` methods instead of raw `RETRO_ENVIRONMENT_*` command
  numbers.
- `Runtime` helpers instead of direct callback function pointers.
- Typed enums and newtypes (`JoypadButton`, `PixelFormat`, `MemoryRegion`,
  `EmulatedAddress`) instead of `u32` IDs and bitmasks.
- Service interfaces (`VfsInterface`, `MidiInterface`,
  `MicrophoneInterface`, `PerfInterface`, etc.) instead of poking
  callback tables.

## Rules When Adding A New Mapping

A typed feature is correct only if it preserves the upstream contract.
Before adding a builder, check each of these:

| Concern | What to verify |
| --- | --- |
| Pointer lifetimes | Frontend keeps the pointer for how long? Does the wrapper need to retain backing storage (`StringPool`, `SubsystemInfoStorage`, etc.)? |
| Nullable function pointers | Some `retro_*_callback` fields are `Option<unsafe extern "C" fn(...)>`. Treat `None` as "frontend didn't install this". |
| Callback ordering | `RETRO_ENVIRONMENT_SET_*_CALLBACK` calls usually have to land *before* the corresponding feature is used, sometimes only between specific lifecycle events. |
| Retained strings | Strings passed to the frontend must outlive whatever struct holds the pointer. Use the retained `CString` storage patterns already in `options.rs` and `subsystem.rs`. |
| Multi-step contracts | Some environment commands negotiate in two passes (request → frontend writes back). Wrap both steps in one typed method. |

When in doubt, mirror an existing wrapper that handles the same shape:
`set_subsystem_info` for retained slices, `set_core_options_v2` for
retained nested structures, `vfs_interface` for RAII handles, and
`set_message_ext` for typed enum payloads.

## What Lives Where

| Module | Purpose |
| --- | --- |
| `crates/libretro-core/src/raw.rs` | Auto-generated bindings to `libretro.h`. |
| `crates/libretro-core/src/glow_context.rs` | Frontend procedure loading for standard glow. |
| `crates/libretro-core/libretro_coverage.md` | Tracker mapping `libretro.h` categories to typed coverage. |

The coverage tracker is the right place to look before adding a new
typed feature — it shows what is already wrapped, what is partially
wrapped, and what is still raw-only.
