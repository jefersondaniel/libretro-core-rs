#[cfg(test)]
use crate::HwContextType;
use crate::HwRenderConfig;
use crate::raw;
use std::ffi::c_void;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HwRenderInterfaceType {
    Vulkan,
    D3d9,
    D3d10,
    D3d11,
    D3d12,
    GskitPs2,
    Unknown(i32),
}

impl HwRenderInterfaceType {
    pub(crate) fn from_raw(raw: i32) -> Self {
        match raw {
            value if value == raw::retro_hw_render_interface_type::Vulkan as i32 => Self::Vulkan,
            value if value == raw::retro_hw_render_interface_type::D3d9 as i32 => Self::D3d9,
            value if value == raw::retro_hw_render_interface_type::D3d10 as i32 => Self::D3d10,
            value if value == raw::retro_hw_render_interface_type::D3d11 as i32 => Self::D3d11,
            value if value == raw::retro_hw_render_interface_type::D3d12 as i32 => Self::D3d12,
            value if value == raw::retro_hw_render_interface_type::GskitPs2 as i32 => {
                Self::GskitPs2
            }
            value => Self::Unknown(value),
        }
    }

    #[cfg(test)]
    pub(crate) fn as_raw(self) -> i32 {
        match self {
            Self::Vulkan => raw::retro_hw_render_interface_type::Vulkan as i32,
            Self::D3d9 => raw::retro_hw_render_interface_type::D3d9 as i32,
            Self::D3d10 => raw::retro_hw_render_interface_type::D3d10 as i32,
            Self::D3d11 => raw::retro_hw_render_interface_type::D3d11 as i32,
            Self::D3d12 => raw::retro_hw_render_interface_type::D3d12 as i32,
            Self::GskitPs2 => raw::retro_hw_render_interface_type::GskitPs2 as i32,
            Self::Unknown(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HwRenderInterface<'a> {
    raw: &'a raw::retro_hw_render_interface,
}

impl<'a> HwRenderInterface<'a> {
    pub(crate) fn from_raw(raw: &'a raw::retro_hw_render_interface) -> Self {
        Self { raw }
    }

    pub fn interface_type(self) -> HwRenderInterfaceType {
        HwRenderInterfaceType::from_raw(self.raw.interface_type)
    }

    pub fn interface_version(self) -> u32 {
        self.raw.interface_version
    }

    pub fn as_base_ptr(self) -> *const c_void {
        (self.raw as *const raw::retro_hw_render_interface).cast::<c_void>()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HwRenderContextNegotiationInterfaceType {
    Vulkan,
    Unknown(i32),
}

impl HwRenderContextNegotiationInterfaceType {
    #[cfg(test)]
    pub(crate) fn from_raw(raw: i32) -> Self {
        match raw {
            value
                if value
                    == raw::retro_hw_render_context_negotiation_interface_type::Vulkan as i32 =>
            {
                Self::Vulkan
            }
            value => Self::Unknown(value),
        }
    }

    pub(crate) fn as_raw(self) -> i32 {
        match self {
            Self::Vulkan => raw::retro_hw_render_context_negotiation_interface_type::Vulkan as i32,
            Self::Unknown(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HwRenderContextNegotiationInterface {
    interface_type: HwRenderContextNegotiationInterfaceType,
    interface_version: u32,
}

impl HwRenderContextNegotiationInterface {
    pub fn new(
        interface_type: HwRenderContextNegotiationInterfaceType,
        interface_version: u32,
    ) -> Self {
        Self {
            interface_type,
            interface_version,
        }
    }

    pub fn vulkan(interface_version: u32) -> Self {
        Self::new(
            HwRenderContextNegotiationInterfaceType::Vulkan,
            interface_version,
        )
    }

    pub fn interface_type(self) -> HwRenderContextNegotiationInterfaceType {
        self.interface_type
    }

    pub fn interface_version(self) -> u32 {
        self.interface_version
    }

    #[cfg(test)]
    pub(crate) fn from_raw(raw: raw::retro_hw_render_context_negotiation_interface) -> Self {
        Self {
            interface_type: HwRenderContextNegotiationInterfaceType::from_raw(raw.interface_type),
            interface_version: raw.interface_version,
        }
    }

    pub(crate) fn as_raw(self) -> raw::retro_hw_render_context_negotiation_interface {
        raw::retro_hw_render_context_negotiation_interface {
            interface_type: self.interface_type.as_raw(),
            interface_version: self.interface_version,
        }
    }
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

    #[test]
    fn hw_render_interface_types_preserve_unknown_values() {
        assert_eq!(
            HwRenderInterfaceType::from_raw(raw::retro_hw_render_interface_type::D3d11 as i32),
            HwRenderInterfaceType::D3d11
        );
        assert_eq!(HwRenderInterfaceType::Unknown(99).as_raw(), 99);
    }

    #[test]
    fn hw_render_context_negotiation_interface_preserves_type_and_version() {
        let interface = HwRenderContextNegotiationInterface::vulkan(2);
        let raw = interface.as_raw();

        assert_eq!(
            raw.interface_type,
            raw::retro_hw_render_context_negotiation_interface_type::Vulkan as i32
        );
        assert_eq!(raw.interface_version, 2);
        assert_eq!(
            HwRenderContextNegotiationInterface::from_raw(raw),
            interface
        );
    }
}
