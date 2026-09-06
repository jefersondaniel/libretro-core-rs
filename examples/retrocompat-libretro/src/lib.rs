//! GLES2-first compatibility example: standard glow, a rotating triangle, and
//! bitmap diagnostics. The frontend owns the context; only callbacks call GL.
use libretro::glow::{self, HasContext};
use libretro::{
    ContentContract, Core, Environment, GameInfo, PixelFormat, Runtime, SystemAvInfo, SystemInfo,
    fixed_system_av_info, opengl_compatibility_hw_render_candidates,
    silent_stereo_frames_for_video_frame,
};
use libretro_diagnostics::{DiagnosticTextLayout, DiagnosticTextOverlay};
use std::time::Instant;
mod perf;
#[path = "../../support/triangle.rs"]
mod renderer;
use perf::{PerfAccumulator, PerfFrameSample, format_perf_lines};
use renderer::TriangleRenderer;
const WIDTH: u32 = 512;
const HEIGHT: u32 = 288;
struct RetrocompatLibretroCore {
    gl: Option<glow::Context>,
    triangle: Option<TriangleRenderer>,
    text: Option<DiagnosticTextOverlay>,
    software: bool,
    framebuffer: Vec<u32>,
    silence: Vec<[i16; 2]>,
    frame: u64,
    perf: PerfAccumulator,
    previous_frame: Option<Instant>,
}
impl Default for RetrocompatLibretroCore {
    fn default() -> Self {
        Self {
            gl: None,
            triangle: None,
            text: None,
            software: false,
            framebuffer: vec![0x00ff0000; (WIDTH * HEIGHT) as usize],
            silence: silent_stereo_frames_for_video_frame(48_000, 60),
            frame: 0,
            perf: PerfAccumulator::default(),
            previous_frame: None,
        }
    }
}
impl Core for RetrocompatLibretroCore {
    fn system_info(&self) -> SystemInfo {
        SystemInfo::new("retrocompat-libretro", env!("CARGO_PKG_VERSION"))
    }
    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(WIDTH, HEIGHT, 60.0, 48_000.0)
    }
    fn on_set_environment(&mut self, env: &mut Environment<'_>) {
        let _ = ContentContract::new("")
            .with_support_no_game(true)
            .register_environment(env);
    }
    fn load_game(&mut self, _: Option<GameInfo<'_>>, rt: &mut Runtime<'_>) -> bool {
        let mut env = rt.environment();
        if !env.set_pixel_format(PixelFormat::Xrgb8888) {
            return false;
        }
        self.software = env
            .set_hw_render_from_candidates(&opengl_compatibility_hw_render_candidates())
            .is_none();
        if self.software {
            let _ = env.set_message(
                "Hardware negotiation rejected; showing software diagnostic",
                360,
            );
        }
        true
    }
    fn hw_context_reset(&mut self, rt: &mut Runtime<'_>) {
        // Abandon names from a lost context. Cleanup on a live retiring context
        // belongs in hw_context_destroy, never here or in unload_game.
        self.text = None;
        self.triangle = None;
        self.gl = None;
        self.frame = 0;
        self.perf = PerfAccumulator::default();
        self.previous_frame = None;
        let gl = match rt.create_glow_context() {
            Ok(gl) => gl,
            Err(e) => {
                rt.logger().error(&e);
                let _ = rt.set_message(e, 360);
                return;
            }
        };
        // SAFETY: reset supplies the current context used by all these objects.
        unsafe {
            match TriangleRenderer::new(&gl) {
                Ok(t) => self.triangle = Some(t),
                Err(e) => {
                    rt.logger().error(&e);
                    let _ = rt.set_message(e, 360);
                }
            }
            let lines = format_perf_lines(None);
            let refs = lines.each_ref().map(String::as_str);
            match DiagnosticTextOverlay::new_with_layout(
                &gl,
                &refs,
                DiagnosticTextLayout::new(12.0, 216.0, 2.0),
            ) {
                Ok(t) => self.text = Some(t),
                Err(e) => {
                    rt.logger().error(&e);
                    let _ = rt.set_message(e, 360);
                }
            }
        }
        rt.logger()
            .info(format!("glow initialized: {:?}", gl.version()));
        self.gl = Some(gl);
    }
    fn hw_context_destroy(&mut self, _: &mut Runtime<'_>) {
        // SAFETY: libretro makes the retiring context current for this callback.
        if let Some(gl) = self.gl.as_ref() {
            unsafe {
                if let Some(text) = self.text.take() {
                    text.destroy(gl);
                }
                if let Some(triangle) = self.triangle.take() {
                    triangle.destroy(gl);
                }
            }
        }
        self.gl = None;
    }
    fn unload_game(&mut self) {
        self.text = None;
        self.triangle = None;
        self.gl = None;
    }
    fn run(&mut self, rt: &mut Runtime<'_>) {
        let start = Instant::now();
        rt.poll_input();
        if self.software {
            let _ = rt.video_refresh_frame_with_audio(
                &self.framebuffer,
                WIDTH,
                HEIGHT,
                WIDTH as usize * 4,
                &self.silence,
            );
            return;
        }
        let (Some(gl), Some(fbo)) = (self.gl.as_ref(), rt.current_framebuffer()) else {
            let _ = rt.video_refresh_dupe_with_audio(WIDTH, HEIGHT, &self.silence);
            return;
        };
        let mut draws = 0;
        // SAFETY: run uses the same current context as reset. All owned objects
        // are discarded on reset/destroy; the framebuffer is queried each frame.
        unsafe {
            if let Some(triangle) = self.triangle.as_ref() {
                let angle = (self.frame % 120) as f32 / 120.0 * std::f32::consts::TAU;
                let mut vertices = [
                    [0.0, 0.62, 0.2, 0.8, 1.0],
                    [-0.62, -0.5, 1.0, 0.35, 0.2],
                    [0.62, -0.5, 1.0, 0.9, 0.15],
                ];
                for v in &mut vertices {
                    let (x, y) = (v[0], v[1]);
                    v[0] = x * angle.cos() - y * angle.sin();
                    v[1] = x * angle.sin() + y * angle.cos();
                }
                match triangle.draw(
                    gl,
                    fbo,
                    WIDTH as i32,
                    HEIGHT as i32,
                    &vertices,
                    [0.08, 0.12, 0.4, 1.0],
                ) {
                    Ok(()) => draws += 1,
                    Err(e) => {
                        let _ = rt.set_message(e, 180);
                    }
                }
            } else {
                gl.bind_framebuffer(
                    glow::FRAMEBUFFER,
                    std::num::NonZeroU32::new(fbo).map(glow::NativeFramebuffer),
                );
                gl.disable(glow::SCISSOR_TEST);
                gl.color_mask(true, true, true, true);
                gl.viewport(0, 0, WIDTH as i32, HEIGHT as i32);
                gl.clear_color(1.0, 0.0, 0.7, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
            }
            if let Some(text) = self.text.as_ref() {
                match text.draw(gl, WIDTH, HEIGHT, [1.0; 4]) {
                    Ok(()) => draws += 1,
                    Err(e) => {
                        let _ = rt.set_message(e, 180);
                    }
                }
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
        let render_submit = start.elapsed();
        let refresh_start = Instant::now();
        rt.video_refresh_hw(WIDTH, HEIGHT, 0);
        let video_refresh = refresh_start.elapsed();
        let audio_start = Instant::now();
        rt.audio_sample_batch(&self.silence);
        let total = self
            .previous_frame
            .replace(start)
            .map(|last| start.duration_since(last))
            .unwrap_or_default();
        if let Some(report) = self.perf.push(PerfFrameSample {
            total,
            render_submit,
            video_refresh,
            audio_flush: audio_start.elapsed(),
            draw_submissions: draws,
            uploads: u32::from(self.triangle.is_some()),
        }) {
            let lines = format_perf_lines(Some(report));
            let refs = lines.each_ref().map(String::as_str);
            if let Some(text) = self.text.as_mut() {
                // SAFETY: still inside the current context's run callback.
                if let Err(e) = unsafe { text.update_lines(gl, &refs) } {
                    let _ = rt.set_message(e, 180);
                }
            }
        }
        self.frame = self.frame.wrapping_add(1);
    }
}
libretro::export_core!(RetrocompatLibretroCore::default());
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audio_batch_matches_frame_rate() {
        let c = RetrocompatLibretroCore::default();
        assert_eq!(c.silence, vec![[0, 0]; 800]);
    }
}
