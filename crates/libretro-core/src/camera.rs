//! Camera capability, request, frame, and service-interface wrappers.
//!
//! Camera callbacks are event-shaped: cores register handlers through
//! `CoreEventConfig`, while camera start/stop capability is exposed through the
//! typed `CameraInterface` service.

use enumflags2::{BitFlags, bitflags};

use crate::raw;

#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CameraCapability {
    OpenGlTexture = 1u64 << (raw::retro_camera_buffer::OpenGlTexture as u64),
    RawFramebuffer = 1u64 << (raw::retro_camera_buffer::RawFramebuffer as u64),
}

pub type CameraCapabilities = BitFlags<CameraCapability>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CameraFrameSize {
    pub width: u32,
    pub height: u32,
}

impl CameraFrameSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn frontend_default() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CameraRequest {
    pub capabilities: CameraCapabilities,
    pub size: CameraFrameSize,
}

impl CameraRequest {
    pub fn raw_framebuffer() -> Self {
        Self {
            capabilities: CameraCapabilities::from(CameraCapability::RawFramebuffer),
            size: CameraFrameSize::frontend_default(),
        }
    }

    pub fn open_gl_texture() -> Self {
        Self {
            capabilities: CameraCapabilities::from(CameraCapability::OpenGlTexture),
            size: CameraFrameSize::frontend_default(),
        }
    }

    pub fn with_size(mut self, size: CameraFrameSize) -> Self {
        self.size = size;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CameraRawFrame<'a> {
    pub pixels: &'a [u32],
    pub width: u32,
    pub height: u32,
    pub pitch_bytes: usize,
}

impl<'a> CameraRawFrame<'a> {
    pub(crate) unsafe fn from_raw(
        buffer: *const u32,
        width: u32,
        height: u32,
        pitch_bytes: usize,
    ) -> Option<Self> {
        if buffer.is_null() {
            return None;
        }
        let words_per_row = pitch_bytes.checked_div(std::mem::size_of::<u32>())?;
        let len = words_per_row.checked_mul(height as usize)?;
        Some(Self {
            pixels: unsafe { std::slice::from_raw_parts(buffer, len) },
            width,
            height,
            pitch_bytes,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CameraTextureId(u32);

impl CameraTextureId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CameraTextureTarget(u32);

impl CameraTextureTarget {
    pub const fn new(target: u32) -> Self {
        Self(target)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraTextureFrame {
    pub texture_id: CameraTextureId,
    pub texture_target: CameraTextureTarget,
    pub affine: [f32; 9],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CameraInterface {
    raw: raw::retro_camera_callback,
}

impl CameraInterface {
    pub(crate) const fn from_raw(raw: raw::retro_camera_callback) -> Self {
        Self { raw }
    }

    pub const fn is_available(self) -> bool {
        self.raw.start.is_some() && self.raw.stop.is_some()
    }

    pub fn start(self) -> bool {
        self.raw.start.is_some_and(|start| unsafe { start() })
    }

    pub fn stop(self) -> bool {
        let Some(stop) = self.raw.stop else {
            return false;
        };
        unsafe { stop() };
        true
    }

    pub fn capabilities(self) -> CameraCapabilities {
        CameraCapabilities::from_bits_truncate(self.raw.caps)
    }

    pub const fn size(self) -> CameraFrameSize {
        CameraFrameSize {
            width: self.raw.width,
            height: self.raw.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CameraCapabilities, CameraCapability, CameraFrameSize, CameraInterface, CameraRequest,
        CameraTextureId, CameraTextureTarget,
    };

    #[test]
    fn camera_capabilities_encode_libretro_bits() {
        let capabilities = CameraCapabilities::from(CameraCapability::RawFramebuffer)
            | CameraCapability::OpenGlTexture;

        assert!(capabilities.contains(CameraCapability::RawFramebuffer));
        assert!(capabilities.contains(CameraCapability::OpenGlTexture));
        assert_eq!(capabilities.bits(), 0b11);
    }

    #[test]
    fn camera_request_preserves_size_hints() {
        let request = CameraRequest::raw_framebuffer().with_size(CameraFrameSize::new(320, 240));

        assert_eq!(request.size, CameraFrameSize::new(320, 240));
        assert!(
            request
                .capabilities
                .contains(CameraCapability::RawFramebuffer)
        );
        assert!(
            CameraRequest::open_gl_texture()
                .capabilities
                .contains(CameraCapability::OpenGlTexture)
        );
    }

    #[test]
    fn empty_camera_interface_reports_unavailable() {
        let camera = CameraInterface::default();

        assert!(!camera.is_available());
        assert!(!camera.start());
        assert!(!camera.stop());
        assert!(camera.capabilities().is_empty());
        assert_eq!(camera.size(), CameraFrameSize::frontend_default());
    }

    #[test]
    fn camera_texture_newtypes_preserve_raw_values() {
        assert_eq!(CameraTextureId::new(7).get(), 7);
        assert_eq!(CameraTextureTarget::new(0x0de1).get(), 0x0de1);
    }
}
