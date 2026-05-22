//! Staged OpenGL diagnostics setup.
//!
//! This module loads enough GL functionality to show progressive visible
//! diagnostics while a hardware-rendered core initializes richer rendering.

use libretro::{Gl, Logger, Runtime};

/// Staged GL symbols for visible libretro diagnostics.
///
/// The clear stage is mandatory because it is the legal post-`SET_HW_RENDER`
/// minimum for visible diagnostics. Triangle/block-text and FNT text symbols
/// are optional layers so product renderer failures do not suppress simpler
/// hardware-frame output.
#[derive(Clone)]
pub struct StagedDiagnosticGl {
    pub gl: Gl,
}

impl StagedDiagnosticGl {
    pub fn init(runtime: &Runtime<'_>, logger: Logger, component: &str) -> Option<Self> {
        let gl = match Gl::init(runtime) {
            Ok(gl) => gl,
            Err(error) => {
                logger.error(format!(
                    "{component}: cannot initialize clear-only diagnostic GL symbols: {error}"
                ));
                return None;
            }
        };

        if !gl.supports_shader_pipeline() {
            logger.warn(format!(
                "{component}: diagnostic GL is limited to clear-only output"
            ));
        } else if !gl.supports_textures() {
            logger.warn(format!(
                "{component}: diagnostic GL cannot render embedded font text"
            ));
        }

        Some(Self { gl })
    }
}
