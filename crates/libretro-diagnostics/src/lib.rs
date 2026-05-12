//! Optional visible diagnostic rendering helpers for libretro cores.
//!
//! This crate intentionally sits above `libretro`: it can decide how to render
//! diagnostic frames, while `libretro` stays focused on ABI/runtime primitives.

mod diagnostic_frame;
mod diagnostic_gl;
mod diagnostic_text;

pub use diagnostic_frame::{
    DiagnosticFrameText, DiagnosticGl, diagnostic_block_text_vertices, render_diagnostic_gl_frame,
    render_software_diagnostic_xrgb8888_frame, wrap_diagnostic_message,
};
pub use diagnostic_gl::StagedDiagnosticGl;
pub use diagnostic_text::{
    DIAGNOSTIC_FONT_BYTES, DIAGNOSTIC_TEXT_FLOATS_PER_VERTEX, DiagnosticFont, DiagnosticGlyph,
    DiagnosticTextLayout, DiagnosticTextOverlay, diagnostic_text_vertices,
    diagnostic_text_vertices_with_layout,
};

#[cfg(test)]
mod test_support {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    pub(crate) fn fake_gl_test_guard() -> MutexGuard<'static, ()> {
        static FAKE_GL_TEST_GUARD: OnceLock<Mutex<()>> = OnceLock::new();
        FAKE_GL_TEST_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("fake GL test guard mutex poisoned")
    }
}
