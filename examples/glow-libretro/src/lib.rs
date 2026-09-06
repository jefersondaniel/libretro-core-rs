//! Minimal hardware core: libretro creates the frontend connection; glow issues
//! GL commands. Build this crate as a cdylib and load it without content.
use libretro::glow::{self, HasContext};
use libretro::{
    ContentContract, Core, Environment, GameInfo, PixelFormat, Runtime, SystemAvInfo, SystemInfo,
    fixed_system_av_info, opengl_compatibility_hw_render_candidates,
};
use std::num::NonZeroU32;

#[derive(Default)]
struct GlowCore {
    gl: Option<glow::Context>,
}

impl Core for GlowCore {
    fn system_info(&self) -> SystemInfo {
        SystemInfo::new("glow-libretro", env!("CARGO_PKG_VERSION"))
    }
    fn av_info(&self) -> SystemAvInfo {
        fixed_system_av_info(320, 240, 60.0, 48_000.0)
    }
    fn on_set_environment(&mut self, env: &mut Environment<'_>) {
        let _ = ContentContract::new("")
            .with_support_no_game(true)
            .register_environment(env);
    }
    fn load_game(&mut self, _: Option<GameInfo<'_>>, rt: &mut Runtime<'_>) -> bool {
        let mut env = rt.environment();
        env.set_pixel_format(PixelFormat::Xrgb8888)
            && env
                .set_hw_render_from_candidates(&opengl_compatibility_hw_render_candidates())
                .is_some()
    }
    fn hw_context_reset(&mut self, rt: &mut Runtime<'_>) {
        // Old resources belong to the previous context. This example creates none
        // and installs no debug callback, so dropping the old glow value calls no GL.
        self.gl = None;
        match rt.create_glow_context() {
            Ok(gl) => self.gl = Some(gl),
            Err(error) => {
                rt.logger().error(&error);
                let _ = rt.set_message(error, 300);
            }
        }
    }
    fn run(&mut self, rt: &mut Runtime<'_>) {
        rt.poll_input();
        let audio = [[0_i16; 2]; 800];
        let (Some(gl), Some(fbo)) = (self.gl.as_ref(), rt.current_framebuffer()) else {
            let _ = rt.video_refresh_dupe_with_audio(320, 240, &audio);
            return;
        };
        // SAFETY: the frontend makes our negotiated context current during run.
        // The framebuffer belongs to that context. No client-memory pointers are used.
        unsafe {
            // Framebuffer zero is valid: glow represents it with None.
            gl.bind_framebuffer(
                glow::FRAMEBUFFER,
                NonZeroU32::new(fbo).map(glow::NativeFramebuffer),
            );
            gl.viewport(0, 0, 320, 240);
            gl.disable(glow::SCISSOR_TEST);
            gl.color_mask(true, true, true, true);
            gl.clear_color(0.1, 0.2, 0.7, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
        let _ = rt.video_refresh_hw_with_audio(320, 240, 0, &audio);
    }
    fn hw_context_destroy(&mut self, _: &mut Runtime<'_>) {
        // Delete your buffers/textures here while the retiring context is current.
        self.gl = None;
    }
    fn unload_game(&mut self) {
        // Do not issue GL calls here: unload is not a current-context guarantee.
        self.gl = None;
    }
}
libretro::export_core!(GlowCore::default());
