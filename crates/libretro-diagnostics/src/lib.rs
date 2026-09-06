//! Optional software diagnostics and a glow bitmap-text overlay.
//!
//! CPU font/layout helpers need no GL context. The hardware overlay uses standard
//! glow and requires a current context for construction, drawing, and destruction.
mod diagnostic_frame;
mod diagnostic_text;
mod overlay;
pub use diagnostic_frame::{
    diagnostic_block_text_vertices, render_software_diagnostic_xrgb8888_frame,
    wrap_diagnostic_message,
};
pub use diagnostic_text::{
    DIAGNOSTIC_FONT_BYTES, DIAGNOSTIC_TEXT_FLOATS_PER_VERTEX, DiagnosticFont, DiagnosticGlyph,
    DiagnosticTextLayout, diagnostic_text_vertices, diagnostic_text_vertices_with_layout,
};
pub use overlay::DiagnosticTextOverlay;
