use crate::raw;
use crate::sanitize_cstring;
use enumflags2::{BitFlags, bitflags};
use std::ffi::CStr;
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VfsInterfaceVersion(u32);

impl VfsInterfaceVersion {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfsFileAccess {
    Read = raw::RETRO_VFS_FILE_ACCESS_READ,
    Write = raw::RETRO_VFS_FILE_ACCESS_WRITE,
    UpdateExisting = raw::RETRO_VFS_FILE_ACCESS_UPDATE_EXISTING,
}

pub type VfsFileAccessFlags = BitFlags<VfsFileAccess>;

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfsFileAccessHint {
    FrequentAccess = raw::RETRO_VFS_FILE_ACCESS_HINT_FREQUENT_ACCESS,
}

pub type VfsFileAccessHints = BitFlags<VfsFileAccessHint>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VfsSeekPosition {
    Start,
    Current,
    End,
}

impl VfsSeekPosition {
    pub(crate) fn as_raw(self) -> i32 {
        match self {
            Self::Start => raw::RETRO_VFS_SEEK_POSITION_START,
            Self::Current => raw::RETRO_VFS_SEEK_POSITION_CURRENT,
            Self::End => raw::RETRO_VFS_SEEK_POSITION_END,
        }
    }
}

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfsStatFlag {
    Valid = raw::RETRO_VFS_STAT_IS_VALID,
    Directory = raw::RETRO_VFS_STAT_IS_DIRECTORY,
    CharacterSpecial = raw::RETRO_VFS_STAT_IS_CHARACTER_SPECIAL,
}

pub type VfsStatFlags = BitFlags<VfsStatFlag>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VfsMetadata {
    pub flags: VfsStatFlags,
    pub size: Option<i32>,
}

#[derive(Clone, Copy, Debug)]
pub struct VfsInterface {
    version: VfsInterfaceVersion,
    raw: raw::retro_vfs_interface,
}

impl VfsInterface {
    pub(crate) fn new(version: VfsInterfaceVersion, raw: raw::retro_vfs_interface) -> Self {
        Self { version, raw }
    }

    pub fn version(self) -> VfsInterfaceVersion {
        self.version
    }

    pub fn open_file(
        self,
        path: &str,
        access: VfsFileAccessFlags,
        hints: VfsFileAccessHints,
    ) -> Option<VfsFile> {
        let open = self.raw.open?;
        let path = sanitize_cstring(path);
        // SAFETY: `path` is a valid C string for the duration of the call.
        let handle = unsafe { open(path.as_ptr(), access.bits(), hints.bits()) };
        if handle.is_null() {
            None
        } else {
            Some(VfsFile {
                handle,
                raw: self.raw,
                closed: false,
                _not_send_sync: PhantomData,
            })
        }
    }

    pub fn remove_file(self, path: &str) -> bool {
        let Some(remove) = self.raw.remove else {
            return false;
        };
        let path = sanitize_cstring(path);
        // SAFETY: `path` is a valid C string for the duration of the call.
        unsafe { remove(path.as_ptr()) == 0 }
    }

    pub fn rename(self, old_path: &str, new_path: &str) -> bool {
        let Some(rename) = self.raw.rename else {
            return false;
        };
        let old_path = sanitize_cstring(old_path);
        let new_path = sanitize_cstring(new_path);
        // SAFETY: Both paths are valid C strings for the duration of the call.
        unsafe { rename(old_path.as_ptr(), new_path.as_ptr()) == 0 }
    }

    pub fn stat(self, path: &str) -> Option<VfsMetadata> {
        let stat = self.raw.stat?;
        let path = sanitize_cstring(path);
        let mut size = 0i32;
        // SAFETY: `path` is a valid C string and `size` is a valid out-param.
        let flags = unsafe { stat(path.as_ptr(), &mut size as *mut i32) };
        if flags == 0 {
            None
        } else {
            Some(VfsMetadata {
                flags: VfsStatFlags::from_bits_truncate(flags as u32),
                size: Some(size),
            })
        }
    }

    pub fn create_dir(self, path: &str) -> bool {
        let Some(mkdir) = self.raw.mkdir else {
            return false;
        };
        let path = sanitize_cstring(path);
        // SAFETY: `path` is a valid C string for the duration of the call.
        unsafe { mkdir(path.as_ptr()) == 0 }
    }

    pub fn open_dir(self, path: &str, include_hidden: bool) -> Option<VfsDirectory> {
        let opendir = self.raw.opendir?;
        let path = sanitize_cstring(path);
        // SAFETY: `path` is a valid C string for the duration of the call.
        let handle = unsafe { opendir(path.as_ptr(), include_hidden) };
        if handle.is_null() {
            None
        } else {
            Some(VfsDirectory {
                handle,
                raw: self.raw,
                closed: false,
                _not_send_sync: PhantomData,
            })
        }
    }
}

#[derive(Debug)]
pub struct VfsFile {
    handle: *mut raw::retro_vfs_file_handle,
    raw: raw::retro_vfs_interface,
    closed: bool,
    _not_send_sync: PhantomData<*mut ()>,
}

impl VfsFile {
    pub fn path(&self) -> Option<String> {
        let get_path = self.raw.get_path?;
        // SAFETY: `self.handle` is live while `self` is not closed.
        let path = unsafe { get_path(self.handle) };
        if path.is_null() {
            None
        } else {
            // SAFETY: Frontend returns a NUL-terminated path owned by the handle.
            Some(
                unsafe { CStr::from_ptr(path) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    pub fn size(&self) -> Option<i64> {
        let size = self.raw.size?;
        // SAFETY: `self.handle` is live while `self` is not closed.
        nonnegative_i64(unsafe { size(self.handle) })
    }

    pub fn truncate(&mut self, length: i64) -> bool {
        let Some(truncate) = self.raw.truncate else {
            return false;
        };
        // SAFETY: `self.handle` is live while `self` is not closed.
        unsafe { truncate(self.handle, length) == 0 }
    }

    pub fn tell(&self) -> Option<i64> {
        let tell = self.raw.tell?;
        // SAFETY: `self.handle` is live while `self` is not closed.
        nonnegative_i64(unsafe { tell(self.handle) })
    }

    pub fn seek(&mut self, offset: i64, position: VfsSeekPosition) -> Option<i64> {
        let seek = self.raw.seek?;
        // SAFETY: `self.handle` is live while `self` is not closed.
        nonnegative_i64(unsafe { seek(self.handle, offset, position.as_raw()) })
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        let read = self.raw.read?;
        // SAFETY: `buffer` is valid for writes of its length.
        nonnegative_i64(unsafe {
            read(
                self.handle,
                buffer.as_mut_ptr().cast::<std::ffi::c_void>(),
                buffer.len() as u64,
            )
        })
        .and_then(|value| usize::try_from(value).ok())
    }

    pub fn write(&mut self, buffer: &[u8]) -> Option<usize> {
        let write = self.raw.write?;
        // SAFETY: `buffer` is valid for reads of its length.
        nonnegative_i64(unsafe {
            write(
                self.handle,
                buffer.as_ptr().cast::<std::ffi::c_void>(),
                buffer.len() as u64,
            )
        })
        .and_then(|value| usize::try_from(value).ok())
    }

    pub fn flush(&mut self) -> bool {
        let Some(flush) = self.raw.flush else {
            return false;
        };
        // SAFETY: `self.handle` is live while `self` is not closed.
        unsafe { flush(self.handle) == 0 }
    }

    pub fn close(mut self) -> bool {
        self.close_inner()
    }

    fn close_inner(&mut self) -> bool {
        if self.closed {
            return true;
        }
        self.closed = true;
        let Some(close) = self.raw.close else {
            return false;
        };
        // SAFETY: We mark the handle closed before calling the frontend so Drop
        // cannot close it twice even if the frontend reports failure.
        unsafe { close(self.handle) == 0 }
    }
}

impl Drop for VfsFile {
    fn drop(&mut self) {
        let _ = self.close_inner();
    }
}

#[derive(Debug)]
pub struct VfsDirectory {
    handle: *mut raw::retro_vfs_dir_handle,
    raw: raw::retro_vfs_interface,
    closed: bool,
    _not_send_sync: PhantomData<*mut ()>,
}

impl VfsDirectory {
    pub fn read_next(&mut self) -> bool {
        let Some(readdir) = self.raw.readdir else {
            return false;
        };
        // SAFETY: `self.handle` is live while `self` is not closed.
        unsafe { readdir(self.handle) }
    }

    pub fn entry_name(&self) -> Option<String> {
        let get_name = self.raw.dirent_get_name?;
        // SAFETY: `self.handle` is live while `self` is not closed.
        let name = unsafe { get_name(self.handle) };
        if name.is_null() {
            None
        } else {
            // SAFETY: The returned name is valid until the next read/close.
            Some(
                unsafe { CStr::from_ptr(name) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    pub fn entry_is_dir(&self) -> bool {
        let Some(is_dir) = self.raw.dirent_is_dir else {
            return false;
        };
        // SAFETY: `self.handle` is live while `self` is not closed.
        unsafe { is_dir(self.handle) }
    }

    pub fn close(mut self) -> bool {
        self.close_inner()
    }

    fn close_inner(&mut self) -> bool {
        if self.closed {
            return true;
        }
        self.closed = true;
        let Some(closedir) = self.raw.closedir else {
            return false;
        };
        // SAFETY: We mark closed before the frontend invalidates the handle.
        unsafe { closedir(self.handle) == 0 }
    }
}

impl Drop for VfsDirectory {
    fn drop(&mut self) {
        let _ = self.close_inner();
    }
}

fn nonnegative_i64(value: i64) -> Option<i64> {
    (value >= 0).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vfs_flags_encode_libretro_values() {
        let access = VfsFileAccessFlags::from(VfsFileAccess::Read)
            | VfsFileAccess::Write
            | VfsFileAccess::UpdateExisting;
        assert_eq!(
            access.bits(),
            raw::RETRO_VFS_FILE_ACCESS_READ_WRITE | raw::RETRO_VFS_FILE_ACCESS_UPDATE_EXISTING
        );
        assert_eq!(
            VfsFileAccessHints::from(VfsFileAccessHint::FrequentAccess).bits(),
            raw::RETRO_VFS_FILE_ACCESS_HINT_FREQUENT_ACCESS
        );
        assert_eq!(
            VfsSeekPosition::End.as_raw(),
            raw::RETRO_VFS_SEEK_POSITION_END
        );
        assert_eq!(
            (VfsStatFlags::from(VfsStatFlag::Valid) | VfsStatFlag::Directory).bits(),
            raw::RETRO_VFS_STAT_IS_VALID | raw::RETRO_VFS_STAT_IS_DIRECTORY
        );
    }
}
