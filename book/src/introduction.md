# Introduction

`libretro-core-rs` is a Rust workspace for writing libretro cores without making
core authors work directly against the C ABI. The public API is intentionally
Rust-first: core code should use enums, newtypes, builders, slices, owned
strings, and `Result`/`Option` values instead of raw callback tables, magic
numbers, and unchecked pointers.

The project has two library crates:

- `libretro-core`, imported as `libretro`, owns the core trait, export macro,
  typed libretro wrappers, runtime helpers, input polling, callback event
  routing, environment commands, and OpenGL symbol wrappers.
- `libretro-diagnostics` owns optional visible diagnostics for cores that want
  clear failure frames instead of silent black screens.

The book is organized around core-author workflows rather than source file
names. Each chapter points back to real code in `examples/` and to precise
Rustdoc entry points when the API details matter.

Useful local references:

- [Libretro spec index](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/README.md)
- [Coverage tracker](https://github.com/jefersondaniel/libretro-core-rs/blob/main/crates/libretro-core/libretro_coverage.md)
- [API design rules](https://github.com/jefersondaniel/libretro-core-rs/blob/main/AGENTS.md)
