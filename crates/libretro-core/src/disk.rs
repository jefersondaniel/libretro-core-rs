//! Disk-control newtypes and retained string helpers.
//!
//! The public `Core` disk methods use `DiskIndex`, `DiskTrayState`, and
//! `DiskControlInterfaceVersion` so media state does not leak raw libretro
//! integer conventions into core logic.

use std::ffi::{CString, c_char};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DiskControlInterfaceVersion(u32);

impl DiskControlInterfaceVersion {
    pub const fn new(version: u32) -> Self {
        Self(version)
    }

    pub const fn get(self) -> u32 {
        self.0
    }

    pub const fn supports_extended(self) -> bool {
        self.0 >= 1
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DiskIndex(u32);

impl DiskIndex {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for DiskIndex {
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DiskTrayState {
    #[default]
    Closed,
    Ejected,
}

impl DiskTrayState {
    pub const fn from_ejected(ejected: bool) -> Self {
        if ejected { Self::Ejected } else { Self::Closed }
    }

    pub const fn is_ejected(self) -> bool {
        matches!(self, Self::Ejected)
    }
}

pub(crate) fn write_frontend_string(value: Option<String>, out: *mut c_char, len: usize) -> bool {
    let Some(value) = value else {
        return false;
    };
    let Some(buffer_len_without_nul) = len.checked_sub(1) else {
        return false;
    };
    if out.is_null() {
        return false;
    }

    let value = crate::sanitize_cstring(value);
    copy_cstring_prefix(&value, out, buffer_len_without_nul);
    true
}

fn copy_cstring_prefix(value: &CString, out: *mut c_char, max_bytes: usize) {
    let bytes = value.as_bytes();
    let copied = bytes.len().min(max_bytes);
    // SAFETY: The caller provides a non-null output buffer with room for
    // `max_bytes + 1` bytes. We copy at most that many bytes and always append NUL.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), out, copied);
        *out.add(copied) = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::{DiskControlInterfaceVersion, DiskIndex, DiskTrayState, write_frontend_string};
    use std::ffi::CStr;

    #[test]
    fn disk_newtypes_preserve_raw_values() {
        assert_eq!(DiskIndex::new(3).as_raw(), 3);
        assert_eq!(DiskControlInterfaceVersion::new(0).get(), 0);
        assert!(!DiskControlInterfaceVersion::new(0).supports_extended());
        assert!(DiskControlInterfaceVersion::new(1).supports_extended());
    }

    #[test]
    fn disk_tray_state_hides_raw_ejected_bool() {
        assert_eq!(DiskTrayState::from_ejected(false), DiskTrayState::Closed);
        assert_eq!(DiskTrayState::from_ejected(true), DiskTrayState::Ejected);
        assert!(!DiskTrayState::Closed.is_ejected());
        assert!(DiskTrayState::Ejected.is_ejected());
    }

    #[test]
    fn frontend_string_copy_is_nul_terminated_and_sanitized() {
        let mut out = [0i8; 8];

        assert!(write_frontend_string(
            Some("disk\0one.cue".to_string()),
            out.as_mut_ptr(),
            out.len()
        ));

        assert_eq!(unsafe { CStr::from_ptr(out.as_ptr()) }, c"diskone");
    }
}
