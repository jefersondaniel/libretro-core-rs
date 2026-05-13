# Diagnostics and Performance

Diagnostics are part of the user experience. A core that fails to initialize GL
or misses a frontend capability should prefer visible messages and diagnostic
frames over a silent black screen.

`libretro-diagnostics` provides:

- software XRGB8888 diagnostic frames,
- staged GL initialization helpers,
- simple bitmap text rendering for hardware cores,
- helpers for wrapping diagnostic text into predictable lines.

Performance helpers live in `libretro-core`:

- `CpuFeatures` exposes frontend CPU capability flags.
- `PerfCounter` wraps retained frontend performance counters.
- `PerfInterface` owns the optional frontend callback table.
- `PerfTick` and `PerfTimeMicros` keep counter units explicit.

The [compatibility OpenGL example](../examples/retrocompat-core.md) combines
diagnostics and performance reporting: initialization stages use distinct clear
colors, and runtime performance samples are shown through a text overlay.
