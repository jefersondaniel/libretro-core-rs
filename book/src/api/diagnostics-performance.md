# Diagnostics and Performance

A core that fails silently is hard to debug. This crate ships two
companion tools:

- `libretro-diagnostics` — visible failure frames, staged GL bring-up, and
  text overlays. Use it so the user sees *why* a core is unhappy instead
  of a black screen.
- The `perf` module inside `libretro-core` — typed access to the frontend
  performance counter interface, plus CPU feature detection.

Both are optional. Add them when a core can fail in ways the user cannot
diagnose otherwise (graphics init, frontend symbol availability, frame
budget) or when you need to measure where time is going.

## Software Diagnostic Frame

`render_software_diagnostic_xrgb8888_frame` fills a CPU-side framebuffer
with a gradient background, border, and wrapped diagnostic text. Use it as
the failure path before hardware rendering has been negotiated:

```rust,ignore
use libretro_diagnostics::render_software_diagnostic_xrgb8888_frame;

fn present_init_failure(&mut self, runtime: &mut Runtime<'_>, frame_index: u64) {
    render_software_diagnostic_xrgb8888_frame(
        &mut self.diagnostic_frame, // Vec<u32>, resized in place
        WIDTH,
        HEIGHT,
        frame_index,
        &["my-core 0.1", "Stage 1: load shaders"],
        "OpenGL is not available. Falling back to software diagnostic.",
    );

    let pitch = WIDTH as usize * core::mem::size_of::<u32>();
    let _ = runtime.video_refresh_frame_with_audio(
        bytemuck::cast_slice(&self.diagnostic_frame),
        WIDTH,
        HEIGHT,
        pitch,
        &self.silence,
    );
}
```

The header lines render at the top of the frame; the message is wrapped
into the body. The function resizes the `Vec<u32>` to match the requested
dimensions, so the same buffer can be reused frame to frame.

`wrap_diagnostic_message(text, max_columns)` produces the same line
wrapping standalone if you need it elsewhere.

## Staged GL Initialization

After hardware rendering has been negotiated, use `StagedDiagnosticGl` to
load just enough GL for a clear-only diagnostic frame. The full `Gl`
facade comes back inside the staged value, and richer renderer setup can
fail independently without taking the diagnostic surface with it.

```rust,ignore
use libretro_diagnostics::StagedDiagnosticGl;

fn hw_context_reset(&mut self, runtime: &mut Runtime<'_>) {
    let logger = runtime.logger();

    let Some(staged) = StagedDiagnosticGl::init(runtime, logger, "my-core") else {
        // Even the minimal clear path is unavailable. Fall back to software.
        self.gl = None;
        return;
    };
    let gl = staged.gl.clone();
    self.gl = Some(gl);

    // Try richer setup next; failures here keep the clear-only path alive.
    match TriangleRenderer::new(self.gl.as_ref().unwrap()) {
        Ok(triangle) => self.triangle = Some(triangle),
        Err(err) => {
            runtime.logger().error(format!("triangle init failed: {err}"));
        }
    }
}
```

`StagedDiagnosticGl::init(runtime, logger, component)` returns `None` if
the mandatory clear/framebuffer/viewport symbols cannot load. When it
returns `Some(_)`, the `gl` field is a fully loaded `Gl` and the optional
shader/buffer/texture symbols can be probed with the usual
`gl.supports_*()` checks.

## Text Overlay

`DiagnosticTextOverlay` renders bitmap text into a hardware frame using
shader and texture symbols. Build it once when richer GL features are
available, then update its lines and draw each frame:

```rust,ignore
use libretro_diagnostics::{
    DiagnosticTextLayout, DiagnosticTextOverlay,
};

if gl.supports_textures() && gl.supports_shader_pipeline() {
    let lines = ["FPS: 60.0", "Frame: 16.67 ms"];
    let layout = DiagnosticTextLayout::new(12.0, 16.0, 1.0);
    self.text_overlay = DiagnosticTextOverlay::new_with_layout(
        &gl, &gl, &lines, layout,
    ).ok();
}

// Each frame, after rendering the main scene:
if let Some(overlay) = self.text_overlay.as_mut() {
    let lines = self.format_perf_lines();
    let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let _ = overlay.update_lines(&gl, &line_refs);
    let _ = overlay.draw(&gl, &gl, WIDTH, HEIGHT, [1.0, 0.91, 0.35, 1.0]);
}
```

`DiagnosticTextLayout::DEFAULT` is `(12.0, 16.0, 1.0)`; `new(x, y, scale)`
overrides the position and scale. `draw()` takes RGBA in the [0.0, 1.0]
range. Call `overlay.destroy(&gl, &gl)` in `hw_context_destroy` to free
GPU resources before the context goes away.

Two GL handles are passed (`gl` and `text_gl`). They are usually the same
context but the API leaves room for splitting the overlay into a separate
GL context if a frontend ever requires that.

## CPU Features

`CpuFeatures` is a `BitFlags<CpuFeature>` set queried through
`PerfInterface`:

```rust,ignore
let mut env = runtime.environment();
let Some(perf) = env.perf_interface() else { return; };
let Some(features) = perf.cpu_features() else { return; };

if features.contains(CpuFeature::Avx2) {
    self.audio_path = AudioPath::Avx2;
} else if features.contains(CpuFeature::Sse2) {
    self.audio_path = AudioPath::Sse2;
} else if features.contains(CpuFeature::Neon) {
    self.audio_path = AudioPath::Neon;
} else {
    self.audio_path = AudioPath::Scalar;
}
```

Variants include `Sse`, `Sse2`, `Sse3`, `Ssse3`, `Avx`, `Avx2`, `Neon`,
and several other architecture extensions. Cache the chosen path on the
core struct so the per-frame `run` doesn't re-probe.

## Performance Counters

A `PerfCounter` is a frontend-pinned counter. The lifecycle is
**construct → register → start/stop pairs → read**:

```rust,ignore
use std::pin::Pin;
use libretro::{PerfCounter, PerfInterface};

struct ProfiledCore {
    perf: Option<PerfInterface>,
    cpu_step: Pin<Box<PerfCounter>>,
}

impl Default for ProfiledCore {
    fn default() -> Self {
        Self {
            perf: None,
            cpu_step: PerfCounter::new("my_core_cpu_step"),
        }
    }
}

impl Core for ProfiledCore {
    fn on_set_environment(&mut self, env: &mut Environment<'_>) {
        if let Some(perf) = env.perf_interface() {
            let registered = perf.register_counter(self.cpu_step.as_mut());
            if registered {
                self.perf = Some(perf);
            }
        }
    }

    fn run(&mut self, runtime: &mut Runtime<'_>) {
        if let Some(perf) = self.perf.as_ref() {
            let _ = perf.start_counter(self.cpu_step.as_mut());
        }
        self.advance_one_frame();
        if let Some(perf) = self.perf.as_ref() {
            let _ = perf.stop_counter(self.cpu_step.as_mut());
        }

        let total_ticks = self.cpu_step.total().as_ticks();
        let call_count = self.cpu_step.call_count();
        runtime.logger().debug(format!(
            "step: {total_ticks} ticks across {call_count} calls",
        ));
    }
}
```

A few important rules:

- `PerfCounter::new` returns `Pin<Box<Self>>`. The frontend stores the raw
  pointer when the counter is registered, so the value must not move.
- All `register/start/stop_counter` calls take `Pin<&mut PerfCounter>`;
  use `counter.as_mut()` (where `counter` is `Pin<Box<_>>`) to obtain
  that.
- Counters are unitless ticks. Pair them with `PerfTimeMicros` from
  `perf.time_micros()` if a wall-clock measurement is needed.

`PerfInterface::log()` asks the frontend to dump all registered counters
to its log target — useful from a debug shortcut or `unload_game`.

## When To Use Each Tool

| Situation | Tool |
| --- | --- |
| Hardware negotiation rejected | Software diagnostic frame + frontend message. |
| GL context exists but advanced symbols missing | `StagedDiagnosticGl` + clear-only HW frame. |
| Need to surface per-frame timing | `DiagnosticTextOverlay` driven by `PerfCounter` totals. |
| SIMD path selection | `CpuFeatures` once at setup, cached on the core. |
| Locating slow code | `PerfCounter` start/stop around suspected blocks. |

For a complete end-to-end example combining staged GL init, software
fallback, diagnostic text, and live perf counters, read the
[Compatibility OpenGL core](../examples/retrocompat-core.md) walkthrough
and source.
