use enumflags2::{BitFlags, bitflags};
use std::ffi::{CStr, c_char, c_void};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::slice;

use crate::raw::{
    RETRO_MEMDESC_ALIGN_2, RETRO_MEMDESC_ALIGN_4, RETRO_MEMDESC_ALIGN_8, RETRO_MEMDESC_BIGENDIAN,
    RETRO_MEMDESC_CONST, RETRO_MEMDESC_MINSIZE_2, RETRO_MEMDESC_MINSIZE_4, RETRO_MEMDESC_MINSIZE_8,
    RETRO_MEMDESC_SAVE_RAM, RETRO_MEMDESC_SYSTEM_RAM, RETRO_MEMDESC_VIDEO_RAM,
    RETRO_MEMORY_ACCESS_READ, RETRO_MEMORY_ACCESS_WRITE, RETRO_MEMORY_MASK, RETRO_MEMORY_ROM,
    RETRO_MEMORY_RTC, RETRO_MEMORY_SAVE_RAM, RETRO_MEMORY_SYSTEM_RAM, RETRO_MEMORY_TYPE_CACHED,
    RETRO_MEMORY_VIDEO_RAM, RETRO_SERIALIZATION_QUIRK_CORE_VARIABLE_SIZE,
    RETRO_SERIALIZATION_QUIRK_ENDIAN_DEPENDENT, RETRO_SERIALIZATION_QUIRK_FRONT_VARIABLE_SIZE,
    RETRO_SERIALIZATION_QUIRK_INCOMPLETE, RETRO_SERIALIZATION_QUIRK_MUST_INITIALIZE,
    RETRO_SERIALIZATION_QUIRK_PLATFORM_DEPENDENT, RETRO_SERIALIZATION_QUIRK_SINGLE_SESSION,
    retro_framebuffer, retro_game_info_ext, retro_pixel_format,
};

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FramebufferMemoryAccess {
    Write = RETRO_MEMORY_ACCESS_WRITE,
    Read = RETRO_MEMORY_ACCESS_READ,
}

pub type FramebufferMemoryAccessFlags = BitFlags<FramebufferMemoryAccess>;

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FramebufferMemoryType {
    Cached = RETRO_MEMORY_TYPE_CACHED,
}

pub type FramebufferMemoryTypes = BitFlags<FramebufferMemoryType>;

#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MemoryDescriptorFlag {
    Constant = RETRO_MEMDESC_CONST,
    BigEndian = RETRO_MEMDESC_BIGENDIAN,
    SystemRam = RETRO_MEMDESC_SYSTEM_RAM,
    SaveRam = RETRO_MEMDESC_SAVE_RAM,
    VideoRam = RETRO_MEMDESC_VIDEO_RAM,
}

pub type MemoryDescriptorFlags = BitFlags<MemoryDescriptorFlag>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MemoryDescriptorAlignment {
    TwoBytes,
    FourBytes,
    EightBytes,
}

impl MemoryDescriptorAlignment {
    pub const fn as_raw_flag(self) -> u64 {
        match self {
            Self::TwoBytes => RETRO_MEMDESC_ALIGN_2,
            Self::FourBytes => RETRO_MEMDESC_ALIGN_4,
            Self::EightBytes => RETRO_MEMDESC_ALIGN_8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MemoryDescriptorMinAccessSize {
    TwoBytes,
    FourBytes,
    EightBytes,
}

impl MemoryDescriptorMinAccessSize {
    pub const fn as_raw_flag(self) -> u64 {
        match self {
            Self::TwoBytes => RETRO_MEMDESC_MINSIZE_2,
            Self::FourBytes => RETRO_MEMDESC_MINSIZE_4,
            Self::EightBytes => RETRO_MEMDESC_MINSIZE_8,
        }
    }
}

#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SerializationQuirk {
    Incomplete = RETRO_SERIALIZATION_QUIRK_INCOMPLETE,
    MustInitialize = RETRO_SERIALIZATION_QUIRK_MUST_INITIALIZE,
    CoreVariableSize = RETRO_SERIALIZATION_QUIRK_CORE_VARIABLE_SIZE,
    FrontendVariableSize = RETRO_SERIALIZATION_QUIRK_FRONT_VARIABLE_SIZE,
    SingleSession = RETRO_SERIALIZATION_QUIRK_SINGLE_SESSION,
    EndianDependent = RETRO_SERIALIZATION_QUIRK_ENDIAN_DEPENDENT,
    PlatformDependent = RETRO_SERIALIZATION_QUIRK_PLATFORM_DEPENDENT,
}

pub type SerializationQuirks = BitFlags<SerializationQuirk>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtendedGameInfo<'a> {
    pub full_path: Option<&'a CStr>,
    pub archive_path: Option<&'a CStr>,
    pub archive_file: Option<&'a CStr>,
    pub dir: Option<&'a CStr>,
    pub name: Option<&'a CStr>,
    pub extension: Option<&'a CStr>,
    pub meta: Option<&'a CStr>,
    pub data: Option<&'a [u8]>,
    pub file_in_archive: bool,
    pub persistent_data: bool,
}

impl<'a> ExtendedGameInfo<'a> {
    pub(crate) unsafe fn from_raw(raw: &'a retro_game_info_ext) -> Self {
        Self {
            full_path: unsafe { cstr_from_ptr(raw.full_path) },
            archive_path: unsafe { cstr_from_ptr(raw.archive_path) },
            archive_file: unsafe { cstr_from_ptr(raw.archive_file) },
            dir: unsafe { cstr_from_ptr(raw.dir) },
            name: unsafe { cstr_from_ptr(raw.name) },
            extension: unsafe { cstr_from_ptr(raw.ext) },
            meta: unsafe { cstr_from_ptr(raw.meta) },
            data: if raw.data.is_null() {
                None
            } else {
                // SAFETY: The frontend provides a valid content buffer for the
                // lifetime described by `persistent_data`.
                Some(unsafe { slice::from_raw_parts(raw.data.cast::<u8>(), raw.size) })
            },
            file_in_archive: raw.file_in_archive,
            persistent_data: raw.persistent_data,
        }
    }
}

unsafe fn cstr_from_ptr<'a>(ptr: *const c_char) -> Option<&'a CStr> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: libretro path/meta fields are NUL-terminated strings when non-null.
        Some(unsafe { CStr::from_ptr(ptr) })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SoftwareFramebufferRequest {
    pub width: u32,
    pub height: u32,
    pub access: FramebufferMemoryAccessFlags,
}

impl SoftwareFramebufferRequest {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            access: FramebufferMemoryAccessFlags::from(FramebufferMemoryAccess::Write),
        }
    }

    pub fn with_access(mut self, access: FramebufferMemoryAccessFlags) -> Self {
        self.access = access;
        self
    }

    pub(crate) fn into_raw(self) -> retro_framebuffer {
        retro_framebuffer {
            width: self.width,
            height: self.height,
            access_flags: self.access.bits(),
            ..retro_framebuffer::default()
        }
    }
}

#[derive(Debug)]
pub struct SoftwareFramebuffer {
    data: NonNull<c_void>,
    width: u32,
    height: u32,
    pitch: usize,
    format: retro_pixel_format,
    access: FramebufferMemoryAccessFlags,
    memory: FramebufferMemoryTypes,
}

impl SoftwareFramebuffer {
    pub(crate) fn from_raw(framebuffer: retro_framebuffer) -> Option<Self> {
        Some(Self {
            data: NonNull::new(framebuffer.data)?,
            width: framebuffer.width,
            height: framebuffer.height,
            pitch: framebuffer.pitch,
            format: framebuffer.format,
            access: FramebufferMemoryAccessFlags::from_bits_truncate(framebuffer.access_flags),
            memory: FramebufferMemoryTypes::from_bits_truncate(framebuffer.memory_flags),
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn pitch(&self) -> usize {
        self.pitch
    }

    pub const fn format(&self) -> retro_pixel_format {
        self.format
    }

    pub const fn access(&self) -> FramebufferMemoryAccessFlags {
        self.access
    }

    pub const fn memory(&self) -> FramebufferMemoryTypes {
        self.memory
    }

    pub fn bytes(&self) -> Option<&[u8]> {
        if !self.access.contains(FramebufferMemoryAccess::Read) {
            return None;
        }
        let len = self.byte_len()?;
        // SAFETY: libretro guarantees the returned pointer is valid during the
        // current retro_run iteration for the access flags requested by the core.
        Some(unsafe { slice::from_raw_parts(self.data.as_ptr().cast::<u8>(), len) })
    }

    pub fn bytes_mut(&mut self) -> Option<&mut [u8]> {
        if !self.access.contains(FramebufferMemoryAccess::Write) {
            return None;
        }
        let len = self.byte_len()?;
        // SAFETY: libretro guarantees the returned pointer is valid during the
        // current retro_run iteration for the access flags requested by the core.
        Some(unsafe { slice::from_raw_parts_mut(self.data.as_ptr().cast::<u8>(), len) })
    }

    pub(crate) fn video_refresh_args(&self) -> (*const c_void, u32, u32, usize) {
        (
            self.data.as_ptr().cast_const(),
            self.width,
            self.height,
            self.pitch,
        )
    }

    fn byte_len(&self) -> Option<usize> {
        self.pitch.checked_mul(self.height as usize)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MemoryMapOffset(usize);

impl MemoryMapOffset {
    pub const fn new(offset: usize) -> Self {
        Self(offset)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for MemoryMapOffset {
    fn from(offset: usize) -> Self {
        Self::new(offset)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EmulatedAddress(usize);

impl EmulatedAddress {
    pub const fn new(address: usize) -> Self {
        Self(address)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for EmulatedAddress {
    fn from(address: usize) -> Self {
        Self::new(address)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MemoryMapMask(usize);

impl MemoryMapMask {
    pub const fn new(mask: usize) -> Self {
        Self(mask)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for MemoryMapMask {
    fn from(mask: usize) -> Self {
        Self::new(mask)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MemoryMapLen(usize);

impl MemoryMapLen {
    pub const fn new(len: usize) -> Self {
        Self(len)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for MemoryMapLen {
    fn from(len: usize) -> Self {
        Self::new(len)
    }
}

#[derive(Debug)]
pub struct MemoryMapDescriptor<'a> {
    pub flags: MemoryDescriptorFlags,
    pub alignment: Option<MemoryDescriptorAlignment>,
    pub min_access_size: Option<MemoryDescriptorMinAccessSize>,
    pub ptr: Option<NonNull<u8>>,
    pub offset: MemoryMapOffset,
    pub start: EmulatedAddress,
    pub select: MemoryMapMask,
    pub disconnect: MemoryMapMask,
    pub len: MemoryMapLen,
    pub addrspace: Option<String>,
    _lifetime: PhantomData<&'a mut [u8]>,
}

impl<'a> MemoryMapDescriptor<'a> {
    pub fn new_inaccessible(
        addrspace: impl Into<Option<String>>,
        start: impl Into<EmulatedAddress>,
        select: impl Into<MemoryMapMask>,
    ) -> Self {
        Self {
            flags: MemoryDescriptorFlags::empty(),
            alignment: None,
            min_access_size: None,
            ptr: None,
            offset: MemoryMapOffset::new(0),
            start: start.into(),
            select: select.into(),
            disconnect: MemoryMapMask::zero(),
            len: MemoryMapLen::new(0),
            addrspace: addrspace.into(),
            _lifetime: PhantomData,
        }
    }

    pub fn from_slice(
        addrspace: impl Into<Option<String>>,
        start: impl Into<EmulatedAddress>,
        memory: &'a mut [u8],
    ) -> Self {
        Self {
            flags: MemoryDescriptorFlags::empty(),
            alignment: None,
            min_access_size: None,
            ptr: NonNull::new(memory.as_mut_ptr()),
            offset: MemoryMapOffset::new(0),
            start: start.into(),
            select: MemoryMapMask::zero(),
            disconnect: MemoryMapMask::zero(),
            len: MemoryMapLen::new(memory.len()),
            addrspace: addrspace.into(),
            _lifetime: PhantomData,
        }
    }

    pub fn with_flags(mut self, flags: MemoryDescriptorFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_alignment(mut self, alignment: MemoryDescriptorAlignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    pub fn with_min_access_size(mut self, size: MemoryDescriptorMinAccessSize) -> Self {
        self.min_access_size = Some(size);
        self
    }

    pub(crate) fn raw_flags(&self) -> u64 {
        self.flags.bits()
            | self
                .alignment
                .map(MemoryDescriptorAlignment::as_raw_flag)
                .unwrap_or(0)
            | self
                .min_access_size
                .map(MemoryDescriptorMinAccessSize::as_raw_flag)
                .unwrap_or(0)
    }

    pub fn with_offset(mut self, offset: impl Into<MemoryMapOffset>) -> Self {
        self.offset = offset.into();
        self
    }

    pub fn with_select(mut self, select: impl Into<MemoryMapMask>) -> Self {
        self.select = select.into();
        self
    }

    pub fn with_disconnect(mut self, disconnect: impl Into<MemoryMapMask>) -> Self {
        self.disconnect = disconnect.into();
        self
    }

    pub fn with_len(mut self, len: impl Into<MemoryMapLen>) -> Self {
        self.len = len.into();
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SavestateContext {
    #[default]
    Normal,
    RunaheadSameInstance,
    RunaheadSameBinary,
    RollbackNetplay,
    Unknown(i32),
}

impl SavestateContext {
    pub const fn from_raw(context: i32) -> Self {
        match context {
            0 => Self::Normal,
            1 => Self::RunaheadSameInstance,
            2 => Self::RunaheadSameBinary,
            3 => Self::RollbackNetplay,
            other => Self::Unknown(other),
        }
    }

    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Normal => 0,
            Self::RunaheadSameInstance => 1,
            Self::RunaheadSameBinary => 2,
            Self::RollbackNetplay => 3,
            Self::Unknown(context) => context,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MemoryRegion {
    SaveRam,
    Rtc,
    SystemRam,
    VideoRam,
    Rom,
    Unknown(u32),
}

impl MemoryRegion {
    pub const fn from_raw(id: u32) -> Self {
        match id & RETRO_MEMORY_MASK {
            RETRO_MEMORY_SAVE_RAM => Self::SaveRam,
            RETRO_MEMORY_RTC => Self::Rtc,
            RETRO_MEMORY_SYSTEM_RAM => Self::SystemRam,
            RETRO_MEMORY_VIDEO_RAM => Self::VideoRam,
            RETRO_MEMORY_ROM => Self::Rom,
            _ => Self::Unknown(id),
        }
    }

    pub const fn as_raw(self) -> u32 {
        match self {
            Self::SaveRam => RETRO_MEMORY_SAVE_RAM,
            Self::Rtc => RETRO_MEMORY_RTC,
            Self::SystemRam => RETRO_MEMORY_SYSTEM_RAM,
            Self::VideoRam => RETRO_MEMORY_VIDEO_RAM,
            Self::Rom => RETRO_MEMORY_ROM,
            Self::Unknown(id) => id,
        }
    }
}

#[derive(Debug)]
pub enum CoreMemory<'a> {
    ReadOnly(&'a [u8]),
    ReadWrite(&'a mut [u8]),
}

impl<'a> CoreMemory<'a> {
    pub fn read_only(data: &'a [u8]) -> Self {
        Self::ReadOnly(data)
    }

    pub fn read_write(data: &'a mut [u8]) -> Self {
        Self::ReadWrite(data)
    }

    pub fn len(&self) -> usize {
        match self {
            Self::ReadOnly(data) => data.len(),
            Self::ReadWrite(data) => data.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::ReadOnly(data) => data,
            Self::ReadWrite(data) => data,
        }
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut c_void {
        if self.is_empty() {
            return std::ptr::null_mut();
        }
        match self {
            Self::ReadOnly(data) => data.as_ptr().cast_mut().cast::<c_void>(),
            Self::ReadWrite(data) => data.as_mut_ptr().cast::<c_void>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CoreMemory, FramebufferMemoryAccess, FramebufferMemoryAccessFlags, FramebufferMemoryType,
        FramebufferMemoryTypes, MemoryDescriptorAlignment, MemoryDescriptorFlag,
        MemoryDescriptorFlags, MemoryDescriptorMinAccessSize, MemoryMapDescriptor, MemoryMapLen,
        MemoryMapMask, MemoryMapOffset, MemoryRegion, SavestateContext, SerializationQuirk,
        SerializationQuirks,
    };

    #[test]
    fn known_memory_regions_round_trip_to_libretro_ids() {
        let regions = [
            MemoryRegion::SaveRam,
            MemoryRegion::Rtc,
            MemoryRegion::SystemRam,
            MemoryRegion::VideoRam,
            MemoryRegion::Rom,
        ];

        for region in regions {
            assert_eq!(MemoryRegion::from_raw(region.as_raw()), region);
        }
    }

    #[test]
    fn unknown_memory_region_preserves_original_id() {
        assert_eq!(MemoryRegion::from_raw(99), MemoryRegion::Unknown(99));
        assert_eq!(MemoryRegion::Unknown(99).as_raw(), 99);
    }

    #[test]
    fn memory_region_masks_libretro_memory_type_bits() {
        let id_with_extra_bits = crate::raw::RETRO_MEMORY_SAVE_RAM | 0x100;

        assert_eq!(
            MemoryRegion::from_raw(id_with_extra_bits),
            MemoryRegion::SaveRam
        );
    }

    #[test]
    fn core_memory_wraps_readonly_and_readwrite_slices() {
        let readonly = [1, 2, 3];
        let readonly_memory = CoreMemory::read_only(&readonly);
        assert_eq!(readonly_memory.len(), 3);
        assert_eq!(readonly_memory.as_slice(), &[1, 2, 3]);

        let mut readwrite = [4, 5];
        let mut readwrite_memory = CoreMemory::read_write(&mut readwrite);
        assert_eq!(readwrite_memory.len(), 2);
        assert_eq!(readwrite_memory.as_slice(), &[4, 5]);
        assert!(!readwrite_memory.as_mut_ptr().is_null());

        let mut empty = CoreMemory::read_only(&[]);
        assert!(empty.as_mut_ptr().is_null());
    }

    #[test]
    fn savestate_contexts_round_trip_to_libretro_ids() {
        let contexts = [
            SavestateContext::Normal,
            SavestateContext::RunaheadSameInstance,
            SavestateContext::RunaheadSameBinary,
            SavestateContext::RollbackNetplay,
        ];

        for context in contexts {
            assert_eq!(SavestateContext::from_raw(context.as_raw()), context);
        }
    }

    #[test]
    fn unknown_savestate_context_preserves_original_id() {
        assert_eq!(
            SavestateContext::from_raw(i32::MAX),
            SavestateContext::Unknown(i32::MAX)
        );
        assert_eq!(SavestateContext::Unknown(99).as_raw(), 99);
    }

    #[test]
    fn serialization_quirks_encode_libretro_flags() {
        let quirks = SerializationQuirks::from(SerializationQuirk::MustInitialize)
            | SerializationQuirk::PlatformDependent;

        assert!(quirks.contains(SerializationQuirk::MustInitialize));
        assert!(quirks.contains(SerializationQuirk::PlatformDependent));
        assert_eq!(
            quirks.bits(),
            crate::raw::RETRO_SERIALIZATION_QUIRK_MUST_INITIALIZE
                | crate::raw::RETRO_SERIALIZATION_QUIRK_PLATFORM_DEPENDENT
        );
    }

    #[test]
    fn memory_descriptor_flags_encode_libretro_bits() {
        let flags = MemoryDescriptorFlags::from(MemoryDescriptorFlag::Constant)
            | MemoryDescriptorFlag::BigEndian
            | MemoryDescriptorFlag::SaveRam;

        assert_eq!(
            flags.bits(),
            crate::raw::RETRO_MEMDESC_CONST
                | crate::raw::RETRO_MEMDESC_BIGENDIAN
                | crate::raw::RETRO_MEMDESC_SAVE_RAM
        );
    }

    #[test]
    fn memory_descriptor_alignment_and_min_size_are_exclusive_values() {
        assert_eq!(
            MemoryDescriptorAlignment::TwoBytes.as_raw_flag(),
            crate::raw::RETRO_MEMDESC_ALIGN_2
        );
        assert_eq!(
            MemoryDescriptorAlignment::FourBytes.as_raw_flag(),
            crate::raw::RETRO_MEMDESC_ALIGN_4
        );
        assert_eq!(
            MemoryDescriptorAlignment::EightBytes.as_raw_flag(),
            crate::raw::RETRO_MEMDESC_ALIGN_8
        );
        assert_eq!(
            MemoryDescriptorMinAccessSize::TwoBytes.as_raw_flag(),
            crate::raw::RETRO_MEMDESC_MINSIZE_2
        );
        assert_eq!(
            MemoryDescriptorMinAccessSize::FourBytes.as_raw_flag(),
            crate::raw::RETRO_MEMDESC_MINSIZE_4
        );
        assert_eq!(
            MemoryDescriptorMinAccessSize::EightBytes.as_raw_flag(),
            crate::raw::RETRO_MEMDESC_MINSIZE_8
        );
    }

    #[test]
    fn memory_map_descriptor_from_slice_tracks_typed_fields() {
        let mut ram = [0u8; 8];
        let descriptor =
            MemoryMapDescriptor::from_slice(Some("WRAM".to_string()), 0x7e0000usize, &mut ram)
                .with_flags(MemoryDescriptorFlags::from(MemoryDescriptorFlag::SystemRam))
                .with_alignment(MemoryDescriptorAlignment::FourBytes)
                .with_min_access_size(MemoryDescriptorMinAccessSize::TwoBytes)
                .with_offset(MemoryMapOffset::new(2))
                .with_select(MemoryMapMask::new(0xff0000))
                .with_disconnect(MemoryMapMask::new(0x80))
                .with_len(MemoryMapLen::new(4));

        assert!(descriptor.ptr.is_some());
        assert_eq!(descriptor.offset, MemoryMapOffset::new(2));
        assert_eq!(descriptor.select, MemoryMapMask::new(0xff0000));
        assert_eq!(descriptor.disconnect, MemoryMapMask::new(0x80));
        assert_eq!(descriptor.len, MemoryMapLen::new(4));
        assert_eq!(
            descriptor.raw_flags(),
            crate::raw::RETRO_MEMDESC_SYSTEM_RAM
                | crate::raw::RETRO_MEMDESC_ALIGN_4
                | crate::raw::RETRO_MEMDESC_MINSIZE_2
        );
    }

    #[test]
    fn framebuffer_memory_flags_encode_libretro_bits() {
        let access = FramebufferMemoryAccessFlags::from(FramebufferMemoryAccess::Read)
            | FramebufferMemoryAccess::Write;
        let types = FramebufferMemoryTypes::from(FramebufferMemoryType::Cached);

        assert_eq!(
            access.bits(),
            crate::raw::RETRO_MEMORY_ACCESS_READ | crate::raw::RETRO_MEMORY_ACCESS_WRITE
        );
        assert_eq!(types.bits(), crate::raw::RETRO_MEMORY_TYPE_CACHED);
    }
}
