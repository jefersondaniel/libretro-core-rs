# Storage, Disks, and VFS

Three storage-related libretro surfaces live in this chapter because they
have different ownership models:

- **Disk control** is **core-owned** media state. The core decides what
  "the current disk" means and answers frontend queries.
- **Subsystems** describe **load-time** multi-ROM contracts. Registered
  once during environment setup.
- **VFS** is **frontend-owned** file access. The frontend exposes a
  filesystem interface and the core uses RAII handles to read and write.

Use normal Rust `std::fs` only when the core intentionally bypasses
frontend VFS policy (for example, debug-only paths). Frontend-mediated
paths should always go through `VfsInterface`.

## Disk Control

Implement these `Core` trait methods when the core has swappable disk
images (CD-ROM emulators, multi-disc games):

```rust,ignore
impl Core for MyCore {
    fn disk_image_count(&mut self) -> u32 {
        self.disks.len() as u32
    }

    fn disk_image_index(&mut self) -> DiskIndex {
        DiskIndex::new(self.current_disk)
    }

    fn disk_set_image_index(&mut self, index: DiskIndex) -> bool {
        let i = index.get();
        if (i as usize) < self.disks.len() {
            self.current_disk = i;
            true
        } else {
            false
        }
    }

    fn disk_tray_state(&mut self) -> DiskTrayState {
        if self.tray_open { DiskTrayState::Ejected } else { DiskTrayState::Closed }
    }

    fn disk_set_tray_state(&mut self, state: DiskTrayState) -> bool {
        self.tray_open = matches!(state, DiskTrayState::Ejected);
        true
    }

    fn disk_replace_image_index(
        &mut self,
        index: DiskIndex,
        game: Option<GameInfo<'_>>,
    ) -> bool {
        let i = index.get() as usize;
        match game {
            Some(info) => self.disks[i] = Disk::from_game_info(info),
            None => self.disks.remove(i),
        };
        true
    }

    fn disk_add_image_index(&mut self) -> bool {
        self.disks.push(Disk::empty());
        true
    }

    fn disk_image_path(&mut self, index: DiskIndex) -> Option<String> {
        self.disks.get(index.get() as usize)?.path.clone()
    }

    fn disk_image_label(&mut self, index: DiskIndex) -> Option<String> {
        self.disks.get(index.get() as usize)?.label.clone()
    }
}
```

Register the disk control interface during environment setup. There are
two versions:

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    if let Some(version) = env.disk_control_interface_version() {
        if version.supports_extended() {
            let _ = env.set_disk_control_ext_interface();
        } else {
            let _ = env.set_disk_control_interface();
        }
    } else {
        let _ = env.set_disk_control_interface();
    }
}
```

The extended (v1) interface enables `disk_image_path`, `disk_image_label`,
`disk_set_initial_image`, `disk_add_image_index`, and replace operations.
The v0 interface only supports basic tray/index management.

## Subsystems

A subsystem describes how a core can load multiple related ROMs as a
single "game" — for example, a Super Game Boy core that needs both an SNES
ROM and a Game Boy ROM. Register during environment setup:

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    let sgb = SubsystemInfo::new("Super Game Boy", "sgb", SubsystemId::from(0))
        .with_roms([
            SubsystemRomInfo::new("SNES ROM", "smc|sfc")
                .with_required(true)
                .with_memory([
                    SubsystemMemoryInfo::new("srm", SubsystemMemoryType::from(0x101)),
                ]),
            SubsystemRomInfo::new("Game Boy ROM", "gb|gbc")
                .with_required(true)
                .with_need_fullpath(true)
                .with_memory([
                    SubsystemMemoryInfo::new("sav", SubsystemMemoryType::from(0x101)),
                    SubsystemMemoryInfo::new("rtc", SubsystemMemoryType::from(0x102)),
                ]),
        ]);

    let _ = env.set_subsystem_info(&[sgb]);
}
```

| Type | Purpose |
| --- | --- |
| `SubsystemInfo` | One subsystem entry — description, identifier, numeric id, and the ROMs it loads. |
| `SubsystemRomInfo` | One ROM slot within a subsystem — extensions, fullpath requirements, memory associations. |
| `SubsystemMemoryInfo` | A save/RTC/SRAM file associated with a ROM slot. |
| `SubsystemMemoryType::from(u32)` | Frontend-defined memory category (e.g. `0x101` for cartridge SRAM, `0x102` for RTC). |

The wrapper retains all descriptor strings, so the slice you pass to
`set_subsystem_info` can be built from temporaries.

## VFS

`VfsInterface` is acquired during setup with the version the core needs:

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    self.vfs = env.vfs_interface(VfsInterfaceVersion::new(3));
}
```

### Files

`VfsFile` is RAII — drop closes the file, or call `close()` explicitly to
inspect the result:

```rust,ignore
let Some(vfs) = self.vfs else { return; };

let access = VfsFileAccessFlags::from(VfsFileAccess::Read);
let hints = VfsFileAccessHints::empty();
let Some(mut file) = vfs.open_file("/path/to/save.srm", access, hints) else {
    return;
};

let mut buffer = vec![0u8; 4096];
while let Some(read) = file.read(&mut buffer) {
    if read == 0 { break; }
    self.save_data.extend_from_slice(&buffer[..read]);
}

let _ = file.seek(0, VfsSeekPosition::Start);
let _ = file.tell();
let _ = file.flush();
// Drop closes the file; or:
let _ok = file.close();
```

`VfsFileAccess` variants: `Read`, `Write`, `UpdateExisting`. Combine with
`|` to get multi-mode flags. `VfsSeekPosition` is `Start`, `Current`, or
`End`.

Writing follows the same shape:

```rust,ignore
let access = VfsFileAccessFlags::from(VfsFileAccess::Write)
    | VfsFileAccess::UpdateExisting;
let hints = VfsFileAccessHints::empty();
let Some(mut file) = vfs.open_file("/path/save.srm", access, hints) else {
    return;
};
let _ = file.write(&self.save_data);
let _ = file.flush();
```

### Directories

`VfsDirectory` iterates entries via `read_next`:

```rust,ignore
let Some(mut dir) = vfs.open_dir("/saves", /* include_hidden */ false) else {
    return;
};
while dir.read_next() {
    if let Some(name) = dir.entry_name() {
        let is_dir = dir.entry_is_dir();
        self.entries.push((name, is_dir));
    }
}
```

### Other operations

```rust,ignore
let exists = vfs.stat("/saves/slot1.srm");           // Option<VfsMetadata>
let ok = vfs.create_dir("/saves");
let ok = vfs.rename("/old.srm", "/new.srm");
let ok = vfs.remove_file("/scratch.bin");
```

`VfsMetadata::flags` is a `VfsStatFlags` bitmask (`Valid`, `Directory`,
`CharacterSpecial`); `size` is the file size when known.

`VfsFile` and `VfsDirectory` are `!Send` and `!Sync` — they hold raw
pointers into the frontend callback table and must stay on the thread
that called `run`.

## Reference

- [Memory, Savestates, and Serialization](memory-savestates.md) — for save
  state APIs that often pair with disk control (per-disk save snapshots).
- [Content, AV, and Timing](content-av-timing.md) — primary content
  loading; subsystems extend it for multi-ROM cores.
