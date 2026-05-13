use std::ffi::{CString, c_char};

use crate::raw;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SubsystemId(u32);

impl SubsystemId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for SubsystemId {
    fn from(id: u32) -> Self {
        Self::new(id)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SubsystemMemoryType(u32);

impl SubsystemMemoryType {
    pub const fn new(memory_type: u32) -> Self {
        Self(memory_type)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for SubsystemMemoryType {
    fn from(memory_type: u32) -> Self {
        Self::new(memory_type)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubsystemMemoryInfo {
    pub extension: String,
    pub memory_type: SubsystemMemoryType,
}

impl SubsystemMemoryInfo {
    pub fn new(extension: impl Into<String>, memory_type: impl Into<SubsystemMemoryType>) -> Self {
        Self {
            extension: extension.into(),
            memory_type: memory_type.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubsystemRomInfo {
    pub description: String,
    pub valid_extensions: String,
    pub need_fullpath: bool,
    pub block_extract: bool,
    pub required: bool,
    pub memory: Vec<SubsystemMemoryInfo>,
}

impl SubsystemRomInfo {
    pub fn new(description: impl Into<String>, valid_extensions: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            valid_extensions: valid_extensions.into(),
            need_fullpath: false,
            block_extract: false,
            required: true,
            memory: Vec::new(),
        }
    }

    pub fn with_need_fullpath(mut self, need_fullpath: bool) -> Self {
        self.need_fullpath = need_fullpath;
        self
    }

    pub fn with_block_extract(mut self, block_extract: bool) -> Self {
        self.block_extract = block_extract;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_memory(mut self, memory: impl IntoIterator<Item = SubsystemMemoryInfo>) -> Self {
        self.memory = memory.into_iter().collect();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubsystemInfo {
    pub description: String,
    pub identifier: String,
    pub id: SubsystemId,
    pub roms: Vec<SubsystemRomInfo>,
}

impl SubsystemInfo {
    pub fn new(
        description: impl Into<String>,
        identifier: impl Into<String>,
        id: impl Into<SubsystemId>,
    ) -> Self {
        Self {
            description: description.into(),
            identifier: identifier.into(),
            id: id.into(),
            roms: Vec::new(),
        }
    }

    pub fn with_roms(mut self, roms: impl IntoIterator<Item = SubsystemRomInfo>) -> Self {
        self.roms = roms.into_iter().collect();
        self
    }
}

pub(crate) struct SubsystemInfoStorage {
    pub(crate) _strings: Vec<CString>,
    pub(crate) _memory: Vec<Vec<raw::retro_subsystem_memory_info>>,
    pub(crate) _roms: Vec<Vec<raw::retro_subsystem_rom_info>>,
    pub(crate) raw: Vec<raw::retro_subsystem_info>,
}

impl SubsystemInfoStorage {
    pub(crate) fn new(subsystems: &[SubsystemInfo]) -> Self {
        let mut strings = Vec::new();
        let mut memory = Vec::new();
        let mut roms = Vec::new();
        let mut raw = Vec::with_capacity(subsystems.len() + 1);

        for subsystem in subsystems {
            let desc = push_string(&mut strings, &subsystem.description);
            let ident = push_string(&mut strings, &subsystem.identifier);
            let mut subsystem_roms = Vec::with_capacity(subsystem.roms.len());

            for rom in &subsystem.roms {
                let rom_desc = push_string(&mut strings, &rom.description);
                let valid_extensions = push_string(&mut strings, &rom.valid_extensions);
                let rom_memory = rom
                    .memory
                    .iter()
                    .map(|info| raw::retro_subsystem_memory_info {
                        extension: push_string(&mut strings, &info.extension),
                        memory_type: info.memory_type.as_raw(),
                    })
                    .collect::<Vec<_>>();
                let memory_ptr = ptr_or_null(&rom_memory);
                let num_memory = rom_memory.len() as u32;
                memory.push(rom_memory);
                subsystem_roms.push(raw::retro_subsystem_rom_info {
                    desc: rom_desc,
                    valid_extensions,
                    need_fullpath: rom.need_fullpath,
                    block_extract: rom.block_extract,
                    required: rom.required,
                    memory: memory_ptr,
                    num_memory,
                });
            }

            let roms_ptr = ptr_or_null(&subsystem_roms);
            let num_roms = subsystem_roms.len() as u32;
            roms.push(subsystem_roms);
            raw.push(raw::retro_subsystem_info {
                desc,
                ident,
                roms: roms_ptr,
                num_roms,
                id: subsystem.id.as_raw(),
            });
        }

        raw.push(raw::retro_subsystem_info::default());

        Self {
            _strings: strings,
            _memory: memory,
            _roms: roms,
            raw,
        }
    }
}

fn push_string(strings: &mut Vec<CString>, value: &str) -> *const c_char {
    strings.push(crate::sanitize_cstring(value));
    strings
        .last()
        .expect("pushed subsystem string must be present")
        .as_ptr()
}

fn ptr_or_null<T>(values: &[T]) -> *const T {
    if values.is_empty() {
        std::ptr::null()
    } else {
        values.as_ptr()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SubsystemId, SubsystemInfo, SubsystemInfoStorage, SubsystemMemoryInfo, SubsystemMemoryType,
        SubsystemRomInfo,
    };
    use std::ffi::CStr;

    #[test]
    fn subsystem_storage_retains_strings_and_nested_arrays() {
        let storage = SubsystemInfoStorage::new(&[SubsystemInfo::new("Super Game Boy", "sgb", 7)
            .with_roms([
                SubsystemRomInfo::new("Game Boy ROM", "gb|gbc").with_memory([
                    SubsystemMemoryInfo::new("sav", SubsystemMemoryType::new(0x101)),
                    SubsystemMemoryInfo::new("rtc", 0x102),
                ]),
                SubsystemRomInfo::new("BIOS", "bin")
                    .with_need_fullpath(true)
                    .with_block_extract(true)
                    .with_required(false),
            ])]);

        assert_eq!(storage.raw.len(), 2);
        assert_eq!(storage.raw[0].id, SubsystemId::new(7).as_raw());
        assert_eq!(storage.raw[0].num_roms, 2);
        assert_eq!(storage.raw[1], crate::raw::retro_subsystem_info::default());
        let roms = unsafe {
            std::slice::from_raw_parts(storage.raw[0].roms, storage.raw[0].num_roms as usize)
        };
        assert_eq!(
            unsafe { CStr::from_ptr(storage.raw[0].desc) },
            c"Super Game Boy"
        );
        assert_eq!(unsafe { CStr::from_ptr(storage.raw[0].ident) }, c"sgb");
        assert_eq!(unsafe { CStr::from_ptr(roms[0].desc) }, c"Game Boy ROM");
        assert_eq!(roms[0].num_memory, 2);
        let memory = unsafe { std::slice::from_raw_parts(roms[0].memory, 2) };
        assert_eq!(unsafe { CStr::from_ptr(memory[0].extension) }, c"sav");
        assert_eq!(memory[0].memory_type, 0x101);
        assert_eq!(memory[1].memory_type, 0x102);
        assert!(roms[1].need_fullpath);
        assert!(roms[1].block_extract);
        assert!(!roms[1].required);
        assert!(roms[1].memory.is_null());
        assert_eq!(roms[1].num_memory, 0);
    }
}
