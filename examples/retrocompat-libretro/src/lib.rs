use std::mem;
use std::time::{Duration, Instant};

use libretro::{
    CompatGl, CompatGlClear, CompatTextureGl, ContentContract, Core, GameInfo, GlFramebuffer,
    GlFramebufferTarget, GlRect, OPENGL_COMPATIBILITY_HW_RENDER_LABEL, PixelFormat, Runtime,
    SystemAvInfo, SystemInfo, fixed_system_av_info, opengl_compatibility_hw_render_candidates,
    silent_stereo_frames_for_video_frame,
};
use libretro_diagnostics::{DiagnosticTextLayout, DiagnosticTextOverlay, StagedDiagnosticGl};

use perf::{PerfAccumulator, PerfFrameSample, PerfReport, format_perf_lines};
use renderer::{TriangleInitStage, TriangleRenderer};

mod perf;
mod renderer;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 288;
const FPS_HZ: u32 = 60;
const SAMPLE_RATE_HZ: u32 = 48_000;
const FPS: f64 = FPS_HZ as f64;
const SAMPLE_RATE: f64 = SAMPLE_RATE_HZ as f64;
const CORE_VERSION: &str = "0.1.0";
const SUPPORTED_CONTENT_EXTENSIONS: &str = "bin|dat";
const CLEAR_OK: [f32; 4] = [0.0, 0.18, 0.95, 1.0];
const CLEAR_TRIANGLE_SYMBOL_FAILED: [f32; 4] = [1.0, 0.35, 0.0, 1.0];
const CLEAR_SHADER_PROGRAM_FAILED: [f32; 4] = [0.95, 0.0, 0.95, 1.0];
const CLEAR_ATTRIBUTE_FAILED: [f32; 4] = [1.0, 0.95, 0.0, 1.0];
const CLEAR_TEXT_SETUP_FAILED: [f32; 4] = [0.45, 0.15, 0.95, 1.0];
const TRIANGLE_ROTATION_PERIOD_FRAMES: f32 = 120.0;
const PERF_TEXT_SCALE: f32 = 2.0;
const PERF_TEXT_MARGIN: f32 = 12.0;
const SOFTWARE_HW_NEGOTIATION_REJECTED_0RGB1555: u16 = 0x7c00;
const HW_FRAMEBUFFER_UNAVAILABLE_MESSAGE: &str =
    "retrocompat-libretro: HW accepted, but frontend returned no hardware FBO";
const HW_CONTEXT_RESET_MISSING_MESSAGE: &str =
    "retrocompat-libretro: HW accepted, but GL context_reset did not initialize";

struct RetrocompatLibretroCore {
    clear_gl: Option<CompatGlClear>,
    gl: Option<CompatGl>,
    text_gl: Option<CompatTextureGl>,
    triangle: Option<TriangleRenderer>,
    text: Option<DiagnosticTextOverlay>,
    diagnostic_clear: [f32; 4],
    software_diagnostic: Option<SoftwareDiagnostic>,
    software_framebuffer: Vec<u16>,
    silence: Vec<[i16; 2]>,
    frame_index: u64,
    perf_accumulator: PerfAccumulator,
    last_perf_report: Option<PerfReport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SoftwareDiagnostic {
    HwNegotiationRejected,
}

impl Default for RetrocompatLibretroCore {
    fn default() -> Self {
        Self {
            clear_gl: None,
            gl: None,
            text_gl: None,
            triangle: None,
            text: None,
            diagnostic_clear: CLEAR_TRIANGLE_SYMBOL_FAILED,
            software_diagnostic: None,
            software_framebuffer: vec![0; (WIDTH * HEIGHT) as usize],
            silence: silent_stereo_frames_for_video_frame(SAMPLE_RATE_HZ, FPS_HZ),
            frame_index: 0,
            perf_accumulator: PerfAccumulator::default(),
            last_perf_report: None,
        }
    }
}

impl Core for RetrocompatLibretroCore {
    fn system_info(&self) -> SystemInfo {
        let mut info = SystemInfo::new("retrocompat-libretro", CORE_VERSION);
        content_contract().apply_to_system_info(&mut info);
        info
    }

    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(WIDTH, HEIGHT, FPS, SAMPLE_RATE)
    }

    fn on_set_environment(&mut self, env: &mut libretro::Environment<'_>) {
        let _ = content_contract().register_environment(env);
    }

    fn load_game(&mut self, _game: Option<GameInfo<'_>>, runtime: &mut Runtime<'_>) -> bool {
        let mut env = runtime.environment();
        self.software_diagnostic = None;
        self.frame_index = 0;
        self.perf_accumulator = PerfAccumulator::default();
        self.last_perf_report = None;
        let _ = env.set_pixel_format(PixelFormat::_0Rgb1555);
        let candidates = opengl_compatibility_hw_render_candidates();
        if env.set_hw_render_from_candidates(&candidates).is_some() {
            true
        } else {
            let _ = env.set_message(
                format!(
                    "retrocompat-libretro: frontend rejected {} hardware rendering",
                    OPENGL_COMPATIBILITY_HW_RENDER_LABEL
                ),
                360,
            );
            self.software_diagnostic = Some(SoftwareDiagnostic::HwNegotiationRejected);
            true
        }
    }

    fn unload_game(&mut self) {
        self.destroy_gl_state();
    }

    fn run(&mut self, runtime: &mut Runtime<'_>) {
        let frame_start = Instant::now();
        runtime.poll_input();

        if let Some(diagnostic) = self.software_diagnostic {
            self.present_software_diagnostic(runtime, diagnostic);
            return;
        }

        let Some(framebuffer) = runtime.current_framebuffer() else {
            self.present_hardware_mode_diagnostic(runtime, HW_FRAMEBUFFER_UNAVAILABLE_MESSAGE);
            return;
        };

        if let Some(gl) = self.gl {
            let mut draw_submissions = 0_u32;
            let mut render_error = None;
            let (_, render_submit) = timed(|| {
                if let Err(error) = gl.bind_framebuffer(
                    GlFramebufferTarget::Framebuffer,
                    GlFramebuffer::from_raw(framebuffer),
                ) {
                    render_error = Some(format!("retrocompat-libretro: {error}"));
                    return;
                }
                if let Err(error) = gl.viewport(GlRect::new(0, 0, WIDTH, HEIGHT)) {
                    render_error = Some(format!("retrocompat-libretro: {error}"));
                    return;
                }
                let clear = self.diagnostic_clear;
                gl.clear_color(clear[0], clear[1], clear[2], clear[3]);
                gl.clear_color_depth_buffer();

                if let Some(triangle) = self.triangle.as_ref() {
                    if let Err(error) = triangle.draw(&gl, self.triangle_rotation_radians()) {
                        let _ = runtime.set_message(format!("retrocompat-libretro: {error}"), 180);
                    } else {
                        draw_submissions += 1;
                    }
                    let mut disable_text = None;
                    if let (Some(text), Some(text_gl)) = (self.text.as_ref(), self.text_gl.as_ref())
                    {
                        match text.draw(&gl, text_gl, WIDTH, HEIGHT, [1.0, 1.0, 1.0, 1.0]) {
                            Ok(()) => draw_submissions += 1,
                            Err(error) => disable_text = Some(error),
                        }
                    }
                    if let Some(error) = disable_text {
                        let _ = runtime.set_message(format!("retrocompat-libretro: {error}"), 180);
                        self.disable_text_overlay(&gl);
                    }
                }
                if let Err(error) = gl.check_no_error("retrocompat frame") {
                    let _ = runtime.set_message(format!("retrocompat-libretro: {error}"), 180);
                }

                gl.unbind_framebuffer(GlFramebufferTarget::Framebuffer);
                let _ = gl.check_no_error("retrocompat framebuffer unbind");
            });
            if let Some(error) = render_error {
                self.present_hardware_mode_diagnostic(runtime, &error);
                return;
            }
            let (_, video_refresh) = timed(|| runtime.video_refresh_hw(WIDTH, HEIGHT, 0));
            let (_, audio_flush) = timed(|| {
                let _ = runtime.audio_sample_batch(&self.silence);
            });
            self.record_perf_sample(
                runtime,
                &gl,
                PerfFrameSample {
                    total: frame_start.elapsed(),
                    render_submit,
                    video_refresh,
                    audio_flush,
                    draw_submissions,
                    uploads: 0,
                },
            );
            self.frame_index = self.frame_index.wrapping_add(1);
            return;
        }

        if let Some(clear_gl) = self.clear_gl.as_ref() {
            if let Err(error) = clear_gl.bind_framebuffer(
                GlFramebufferTarget::Framebuffer,
                GlFramebuffer::from_raw(framebuffer),
            ) {
                self.present_hardware_mode_diagnostic(
                    runtime,
                    &format!("retrocompat-libretro: {error}"),
                );
                return;
            }
            if let Err(error) = clear_gl.viewport(GlRect::new(0, 0, WIDTH, HEIGHT)) {
                self.present_hardware_mode_diagnostic(
                    runtime,
                    &format!("retrocompat-libretro: {error}"),
                );
                return;
            }
            let clear = self.diagnostic_clear;
            clear_gl.clear_color(clear[0], clear[1], clear[2], clear[3]);
            clear_gl.clear_color_depth_buffer();
            if let Err(error) = clear_gl.check_no_error("retrocompat clear-only frame") {
                let _ = runtime.set_message(format!("retrocompat-libretro: {error}"), 180);
            }
            clear_gl.unbind_framebuffer(GlFramebufferTarget::Framebuffer);
            let _ = clear_gl.check_no_error("retrocompat clear-only framebuffer unbind");
            let _ = runtime.video_refresh_hw_with_audio(WIDTH, HEIGHT, 0, &self.silence);
            self.frame_index = self.frame_index.wrapping_add(1);
            return;
        }

        self.present_hardware_mode_diagnostic(runtime, HW_CONTEXT_RESET_MISSING_MESSAGE);
    }

    fn hw_context_reset(&mut self, runtime: &mut Runtime<'_>) {
        self.destroy_gl_state();
        self.frame_index = 0;
        self.perf_accumulator = PerfAccumulator::default();
        self.last_perf_report = None;

        let logger = runtime.logger();
        let Some(staged_gl) = StagedDiagnosticGl::init(runtime, logger, "retrocompat-libretro")
        else {
            return;
        };
        let clear_gl = staged_gl.clear;
        self.clear_gl = Some(clear_gl);
        self.diagnostic_clear = CLEAR_TRIANGLE_SYMBOL_FAILED;

        let Some(gl) = staged_gl.gl else {
            logger.error("retrocompat-libretro: failed to initialize GLES2 triangle GL symbols");
            return;
        };
        self.gl = Some(gl);
        self.diagnostic_clear = CLEAR_TRIANGLE_SYMBOL_FAILED;

        self.diagnostic_clear = CLEAR_SHADER_PROGRAM_FAILED;
        let triangle = match TriangleRenderer::new(&gl) {
            Ok(renderer) => renderer,
            Err(error) => {
                self.diagnostic_clear = clear_for_triangle_init_stage(error.stage());
                logger.error(format!("retrocompat-libretro: {}", error.message()));
                return;
            }
        };
        self.triangle = Some(triangle);

        self.diagnostic_clear = CLEAR_TEXT_SETUP_FAILED;
        let Some(text_gl) = staged_gl.text_gl else {
            logger.info(
                "retrocompat-libretro: minimal GLES2 triangle renderer initialized without text overlay",
            );
            return;
        };

        let perf_lines = format_perf_lines(self.last_perf_report);
        let perf_line_refs = perf_line_refs(&perf_lines);
        let text = match DiagnosticTextOverlay::new_with_layout(
            &gl,
            &text_gl,
            &perf_line_refs,
            perf_text_layout(),
        ) {
            Ok(overlay) => overlay,
            Err(error) => {
                logger.error(format!(
                    "retrocompat-libretro: failed to build diagnostic text overlay: {error}"
                ));
                logger.info(
                    "retrocompat-libretro: minimal GLES2 triangle renderer initialized without text overlay",
                );
                return;
            }
        };

        self.text_gl = Some(text_gl);
        self.text = Some(text);
        self.diagnostic_clear = CLEAR_OK;
        logger.info(
            "retrocompat-libretro: rotating GLES2 triangle and performance text renderer initialized",
        );
    }

    fn hw_context_destroy(&mut self, _runtime: &mut Runtime<'_>) {
        self.drop_gl_state_handles();
    }
}

impl RetrocompatLibretroCore {
    fn destroy_gl_state(&mut self) {
        if let Some(gl) = self.gl.as_ref() {
            if let (Some(text), Some(text_gl)) = (self.text.as_mut(), self.text_gl.as_ref()) {
                text.destroy(gl, text_gl);
            }
            if let Some(triangle) = self.triangle.as_mut() {
                triangle.destroy(gl);
            }
        }

        self.drop_gl_state_handles();
    }

    fn drop_gl_state_handles(&mut self) {
        self.clear_gl = None;
        self.gl = None;
        self.text_gl = None;
        self.triangle = None;
        self.text = None;
        self.diagnostic_clear = CLEAR_TRIANGLE_SYMBOL_FAILED;
        self.software_diagnostic = None;
    }

    fn disable_text_overlay(&mut self, gl: &CompatGl) {
        if let (Some(mut text), Some(text_gl)) = (self.text.take(), self.text_gl.as_ref()) {
            text.destroy(gl, text_gl);
        }
        self.text_gl = None;
        self.diagnostic_clear = CLEAR_TEXT_SETUP_FAILED;
    }

    fn triangle_rotation_radians(&self) -> f32 {
        let phase = (self.frame_index % TRIANGLE_ROTATION_PERIOD_FRAMES as u64) as f32
            / TRIANGLE_ROTATION_PERIOD_FRAMES;
        phase * std::f32::consts::TAU
    }

    fn record_perf_sample(
        &mut self,
        runtime: &mut Runtime<'_>,
        gl: &CompatGl,
        sample: PerfFrameSample,
    ) {
        let Some(report) = self.perf_accumulator.push(sample) else {
            return;
        };
        self.last_perf_report = Some(report);
        if let Err(error) = self.update_perf_text_overlay(gl) {
            let _ = runtime.set_message(format!("retrocompat-libretro: {error}"), 180);
            self.disable_text_overlay(gl);
        }
    }

    fn update_perf_text_overlay(&mut self, gl: &CompatGl) -> Result<(), String> {
        let Some(text) = self.text.as_mut() else {
            return Ok(());
        };
        let perf_lines = format_perf_lines(self.last_perf_report);
        let perf_line_refs = perf_line_refs(&perf_lines);
        text.update_lines(gl, &perf_line_refs)
    }

    fn present_software_diagnostic(
        &mut self,
        runtime: &mut Runtime<'_>,
        diagnostic: SoftwareDiagnostic,
    ) {
        self.software_framebuffer
            .fill(software_diagnostic_color(diagnostic));
        let _ = runtime.video_refresh_frame_with_audio(
            &self.software_framebuffer,
            WIDTH,
            HEIGHT,
            WIDTH as usize * mem::size_of::<u16>(),
            &self.silence,
        );
    }

    fn present_hardware_mode_diagnostic(&self, runtime: &mut Runtime<'_>, message: &str) {
        let _ = runtime.set_message(message, 180);
        // Once SET_HW_RENDER succeeds, libretro requires hardware cores to submit
        // either RETRO_HW_FRAME_BUFFER_VALID or NULL. A software framebuffer here is
        // undefined and can preserve the exact black-screen symptom being diagnosed.
        let _ = runtime.video_refresh_dupe_with_audio(WIDTH, HEIGHT, &self.silence);
    }
}

fn clear_for_triangle_init_stage(stage: TriangleInitStage) -> [f32; 4] {
    match stage {
        TriangleInitStage::ShaderProgram => CLEAR_SHADER_PROGRAM_FAILED,
        TriangleInitStage::AttributeOrBuffer => CLEAR_ATTRIBUTE_FAILED,
    }
}

fn software_diagnostic_color(diagnostic: SoftwareDiagnostic) -> u16 {
    match diagnostic {
        SoftwareDiagnostic::HwNegotiationRejected => SOFTWARE_HW_NEGOTIATION_REJECTED_0RGB1555,
    }
}

fn content_contract() -> ContentContract {
    ContentContract::new(SUPPORTED_CONTENT_EXTENSIONS).with_support_no_game(true)
}

fn perf_text_layout() -> DiagnosticTextLayout {
    let line_height = 10.0 * PERF_TEXT_SCALE;
    let y = HEIGHT as f32 - PERF_TEXT_MARGIN - line_height * 3.0;
    DiagnosticTextLayout::new(PERF_TEXT_MARGIN, y.max(PERF_TEXT_MARGIN), PERF_TEXT_SCALE)
}

fn perf_line_refs(lines: &[String; 3]) -> [&str; 3] {
    [lines[0].as_str(), lines[1].as_str(), lines[2].as_str()]
}

fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let value = f();
    (value, start.elapsed())
}

libretro::export_core!(RetrocompatLibretroCore::default());

#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    pub(crate) fn fake_gl_test_guard() -> MutexGuard<'static, ()> {
        static FAKE_GL_TEST_GUARD: OnceLock<Mutex<()>> = OnceLock::new();
        FAKE_GL_TEST_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("fake GL test guard mutex poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libretro::{CompatGl, CompatTextureGl, FakeGlConfig, glsym};

    fn is_bright(channel: f32) -> bool {
        channel > 0.9
    }

    fn is_above(channel: f32, threshold: f32) -> bool {
        channel > threshold
    }

    #[test]
    fn silent_audio_batch_matches_one_frame_at_sixty_hz() {
        let core = RetrocompatLibretroCore::default();

        assert_eq!(core.silence.len(), 800);
        assert!(core.silence.iter().all(|frame| *frame == [0, 0]));
    }

    #[test]
    fn diagnostic_clear_colors_are_bright_enough_to_disambiguate_failures() {
        assert!(is_bright(CLEAR_OK[2]));
        assert!(is_bright(CLEAR_TRIANGLE_SYMBOL_FAILED[0]));
        assert!(is_above(CLEAR_TRIANGLE_SYMBOL_FAILED[1], 0.3));
        assert!(is_bright(CLEAR_SHADER_PROGRAM_FAILED[0]));
        assert!(is_bright(CLEAR_SHADER_PROGRAM_FAILED[2]));
        assert!(is_bright(CLEAR_ATTRIBUTE_FAILED[0]));
        assert!(is_bright(CLEAR_ATTRIBUTE_FAILED[1]));
        assert!(is_bright(CLEAR_TEXT_SETUP_FAILED[2]));
    }

    #[test]
    fn triangle_init_failure_stages_keep_distinct_clear_colors() {
        assert_eq!(
            clear_for_triangle_init_stage(TriangleInitStage::ShaderProgram),
            CLEAR_SHADER_PROGRAM_FAILED
        );
        assert_eq!(
            clear_for_triangle_init_stage(TriangleInitStage::AttributeOrBuffer),
            CLEAR_ATTRIBUTE_FAILED
        );
    }

    #[test]
    fn software_diagnostic_color_marks_pre_hw_negotiation_rejection() {
        let core = RetrocompatLibretroCore::default();

        assert_eq!(core.software_framebuffer.len(), (WIDTH * HEIGHT) as usize);
        assert_eq!(
            software_diagnostic_color(SoftwareDiagnostic::HwNegotiationRejected),
            SOFTWARE_HW_NEGOTIATION_REJECTED_0RGB1555
        );
    }

    #[test]
    fn disabling_text_overlay_keeps_triangle_stage_alive() {
        let _guard = crate::test_support::fake_gl_test_guard();
        glsym::reset_fake_state_for_testing();
        let gl = CompatGl::fake_for_testing(FakeGlConfig::default());
        let text_gl = CompatTextureGl::fake_for_testing(FakeGlConfig::default());
        let perf_lines = format_perf_lines(None);
        let perf_line_refs = perf_line_refs(&perf_lines);
        let text = DiagnosticTextOverlay::new_with_layout(
            &gl,
            &text_gl,
            &perf_line_refs,
            perf_text_layout(),
        )
        .unwrap();
        let mut core = RetrocompatLibretroCore {
            gl: Some(gl),
            text_gl: Some(text_gl),
            text: Some(text),
            diagnostic_clear: CLEAR_OK,
            ..RetrocompatLibretroCore::default()
        };

        core.disable_text_overlay(&gl);

        assert!(core.gl.is_some());
        assert!(core.text.is_none());
        assert!(core.text_gl.is_none());
        assert_eq!(core.diagnostic_clear, CLEAR_TEXT_SETUP_FAILED);
    }

    #[test]
    fn perf_text_layout_uses_bottom_left_scaled_area() {
        let layout = perf_text_layout();

        assert_eq!(layout.x, PERF_TEXT_MARGIN);
        assert_eq!(layout.scale, PERF_TEXT_SCALE);
        assert!(layout.y > HEIGHT as f32 * 0.5);
    }
}
