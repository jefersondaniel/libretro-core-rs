use libretro::{CompatGl, CompatGlClear, CompatTextureGl, Logger, Runtime};

/// Staged GL symbols for visible libretro diagnostics.
///
/// The clear stage is mandatory because it is the legal post-`SET_HW_RENDER`
/// minimum for visible diagnostics. Triangle/block-text and FNT text symbols
/// are optional layers so product renderer failures do not suppress simpler
/// hardware-frame output.
#[derive(Clone, Copy)]
pub struct StagedDiagnosticGl {
    pub clear: CompatGlClear,
    pub gl: Option<CompatGl>,
    pub text_gl: Option<CompatTextureGl>,
}

impl StagedDiagnosticGl {
    pub fn init(runtime: &Runtime<'_>, logger: Logger, component: &str) -> Option<Self> {
        let clear = match CompatGlClear::init(runtime) {
            Ok(clear) => clear,
            Err(error) => {
                logger.error(format!(
                    "{component}: cannot initialize clear-only diagnostic GL symbols: {error}"
                ));
                return None;
            }
        };

        let gl = match CompatGl::init_from_clear(runtime, clear) {
            Ok(gl) => Some(gl),
            Err(error) => {
                logger.warn(format!(
                    "{component}: diagnostic GL is limited to clear-only output: {error}"
                ));
                None
            }
        };

        let text_gl = if gl.is_some() {
            match CompatTextureGl::init(runtime) {
                Ok(text_gl) => Some(text_gl),
                Err(error) => {
                    logger.warn(format!(
                        "{component}: diagnostic GL cannot render embedded font text: {error}"
                    ));
                    None
                }
            }
        } else {
            None
        };

        Some(Self { clear, gl, text_gl })
    }
}
