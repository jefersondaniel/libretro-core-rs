# Raw ABI Boundaries

The raw `libretro.h` mapping exists so the wrapper can preserve ABI contracts
exactly. It is not the normal core-author API.

Use raw names only when auditing the wrapper, writing layout tests, or adding a
typed feature that does not yet exist. Normal cores should prefer:

- `Core` and `export_core!` instead of writing `retro_*` exports.
- `Environment` methods instead of raw `RETRO_ENVIRONMENT_*` commands.
- `Runtime` helpers instead of direct callback function pointers.
- typed enums/newtypes instead of `u32` IDs and bitmasks.
- service interfaces such as `VfsInterface`, `MidiInterface`,
  `MicrophoneInterface`, and `PerfInterface` instead of callback tables.

When adding a new mapping, preserve upstream pointer lifetimes, nullable
function-pointer semantics, callback ordering, retained string storage, and
multi-step contracts before adding ergonomic helpers.
