# Running A Core

A libretro core is not a standalone executable. Cargo builds a dynamic library,
then a frontend loads that library and drives the lifecycle.

For a workspace example:

```sh
cargo build --release -p hello-libretro
```

The output path is platform-specific. On Linux, the hello example builds to a
file like:

```text
target/release/libhello_libretro.so
```

Use the equivalent `.dylib` or `.dll` on other platforms.

The frontend is responsible for loading the core, choosing content or no
content, mapping input devices, opening audio/video devices, and calling the
core repeatedly. After `load_game` succeeds, every frontend call to `run`
should produce a video submission and enough audio to keep pacing stable.

When a core fails to load, prefer a diagnosable path:

- return `false` from `load_game` for invalid required content,
- use `runtime.logger()` or `runtime.set_message(...)` for actionable failures,
- keep hardware-render failures visible with diagnostic frames where possible.

The validation chapter for this workspace is
[Publishing and Validation](reference/publishing.md).
