#[cfg(test)]
use crate::HwContextType;
use crate::HwRenderConfig;

pub const OPENGL_COMPATIBILITY_HW_RENDER_LABEL: &str = "OpenGL/GLESv2/GLES2 candidate";
pub const OPENGL_MODERN_PREFERRED_HW_RENDER_LABEL: &str = "modern-preferred OpenGL/GLES candidate";

pub fn opengl_modern_preferred_hw_render_candidates() -> [HwRenderConfig; 5] {
    [
        // Generic OpenGL is the safest first request on old frontends that
        // lack GET_PREFERRED_HW_RENDER. The active context can still promote
        // to a modern renderer after context reset proves the live features.
        HwRenderConfig::opengl().with_bottom_left_origin(true),
        // Some older RetroArch GL drivers handle explicit GLES 2.0 version
        // requests differently from the legacy OPENGLES2 enum.
        HwRenderConfig::opengles_version(2, 0)
            .with_depth(true)
            .with_bottom_left_origin(true),
        HwRenderConfig::opengles2()
            .with_depth(true)
            .with_bottom_left_origin(true),
        // Strict modern requests are fallbacks for frontends that reject the
        // tolerant families but can satisfy exact modern contexts.
        HwRenderConfig::opengl_core(3, 3).with_bottom_left_origin(true),
        HwRenderConfig::opengles3().with_bottom_left_origin(true),
    ]
}

pub fn opengl_compatibility_hw_render_candidates() -> [HwRenderConfig; 3] {
    [
        // Generic OpenGL can map to a GLES2-class context on some libretro
        // GL frontends; keep it first because the old-device probe proved
        // this request form can present where legacy OPENGLES2 cannot.
        HwRenderConfig::opengl()
            .with_depth(true)
            .with_bottom_left_origin(true),
        // Some older RetroArch GL drivers handle explicit GLES 2.0 version
        // requests differently from the legacy OPENGLES2 enum.
        HwRenderConfig::opengles_version(2, 0)
            .with_depth(true)
            .with_bottom_left_origin(true),
        HwRenderConfig::opengles2()
            .with_depth(true)
            .with_bottom_left_origin(true),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_preferred_candidates_keep_tolerant_paths_before_strict_modern_requests() {
        let candidates = opengl_modern_preferred_hw_render_candidates();

        assert_eq!(candidates[0].context_type, HwContextType::OpenGl);
        assert!(candidates[0].bottom_left_origin);
        assert_eq!(candidates[1].context_type, HwContextType::OpenGlEsVersion);
        assert_eq!(candidates[1].version_major, 2);
        assert_eq!(candidates[1].version_minor, 0);
        assert!(candidates[1].depth);
        assert!(candidates[1].bottom_left_origin);
        assert_eq!(candidates[2].context_type, HwContextType::OpenGlEs2);
        assert!(candidates[2].depth);
        assert!(candidates[2].bottom_left_origin);
        assert_eq!(candidates[3].context_type, HwContextType::OpenGlCore);
        assert_eq!(candidates[3].version_major, 3);
        assert_eq!(candidates[3].version_minor, 3);
        assert!(candidates[3].bottom_left_origin);
        assert_eq!(candidates[4].context_type, HwContextType::OpenGlEs3);
        assert!(candidates[4].bottom_left_origin);
    }

    #[test]
    fn compatibility_candidates_prefer_visible_request_forms_before_legacy_gles2() {
        let candidates = opengl_compatibility_hw_render_candidates();

        assert_eq!(candidates[0].context_type, HwContextType::OpenGl);
        assert!(candidates[0].depth);
        assert!(candidates[0].bottom_left_origin);
        assert_eq!(candidates[1].context_type, HwContextType::OpenGlEsVersion);
        assert_eq!(candidates[1].version_major, 2);
        assert_eq!(candidates[1].version_minor, 0);
        assert!(candidates[1].depth);
        assert!(candidates[1].bottom_left_origin);
        assert_eq!(candidates[2].context_type, HwContextType::OpenGlEs2);
        assert!(candidates[2].depth);
        assert!(candidates[2].bottom_left_origin);
    }
}
