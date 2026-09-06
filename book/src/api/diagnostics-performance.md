# Diagnostics and Performance

`libretro-diagnostics` provides CPU framebuffer/font helpers and a standard glow
bitmap-text overlay. It does not provide a separate GL dispatcher. Performance
counters remain frontend services in `libretro-core`.

## Text overlay

Create the overlay during context reset, update text when it changes, and draw
before submitting the hardware frame. See the [compatibility example](../examples/retrocompat-core.md).

```rust,ignore
use libretro_diagnostics::DiagnosticTextOverlay;
// Inside reset, with a current glow context:
let text = unsafe { DiagnosticTextOverlay::new(&gl, &["Renderer ready"])? };
// Inside run, after binding the target and setting viewport/rasterization state:
unsafe { text.draw(&gl, 320, 240, [1.0; 4])?; }
// Inside destroy, with that same context current:
unsafe { text.destroy(&gl); }
```

| Construct / method | Purpose |
| --- | --- |
| `DiagnosticTextOverlay::new` | Upload the embedded font and create a GPU text pipeline. |
| `new_with_layout` | Choose text position and scale. |
| `update_lines` | Replace text vertices without reuploading the font. |
| `draw` | Draw into the currently bound target; release program, buffer and texture bindings afterward. |
| `destroy` | Consume and delete the overlay while its context is current. |
| `DiagnosticTextLayout::new` | Describe position and scale in pixels. |
| `DiagnosticFont::from_fnt_v1` | Decode font data on the CPU. |
| `diagnostic_text_vertices` / `diagnostic_text_vertices_with_layout` | Generate CPU glyph vertices. |
| `render_software_diagnostic_xrgb8888_frame` | Fill a CPU diagnostic framebuffer before hardware negotiation succeeds. |
| `wrap_diagnostic_message` | Wrap a message into bounded columns. |

GPU methods are unsafe because the caller supplies a glow context and controls
its lifetime. Ordinary Drop makes no GL calls: use it to abandon resources after
context loss. Drawing changes GL state; it does not restore a previous renderer's
state. Establish your next pass's state explicitly.

If glow initialization fails, use `Runtime::set_message`, logging, and duplicate
hardware frames with audio. Do not submit software pixels after hardware mode is
accepted. The old staged partial-symbol dispatcher is removed.

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
