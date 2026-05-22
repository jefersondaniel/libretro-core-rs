# Rustdoc Map

Rustdoc is the precise API reference; this book is the guided manual. Generate
local Rustdoc with:

```sh
cargo doc --workspace --no-deps
```

Primary entry points:

- `libretro::Core`
- `libretro::Runtime`
- `libretro::Environment`
- `libretro::CoreEventConfig`
- `libretro::ContentContract`
- `libretro::SystemInfo`
- `libretro::SystemAvInfo`
- `libretro::CoreOptions`
- `libretro::MemoryRegion`
- `libretro::VfsInterface`
- `libretro::HwRenderConfig`
- `libretro::Gl`
- `libretro_diagnostics::StagedDiagnosticGl`
- `libretro_diagnostics::DiagnosticTextOverlay`

The coverage tracker maps `libretro.h` categories to the Rust API surface:

- [`crates/libretro-core/libretro_coverage.md`](https://github.com/jefersondaniel/libretro-core-rs/blob/main/crates/libretro-core/libretro_coverage.md)
