# Storage, Disks, and VFS

Storage-related libretro APIs have different lifetimes, so this crate exposes
them as separate typed surfaces.

Disk control is core-owned media state. Implement the `Core` disk methods when
a core has swappable images. The public types hide raw indices and tray booleans
behind `DiskIndex`, `DiskTrayState`, and `DiskControlInterfaceVersion`.

Subsystems describe multi-ROM loading contracts. Use `SubsystemInfo`,
`SubsystemRomInfo`, and `SubsystemMemoryInfo` to build retained descriptor
tables without leaking temporary strings to the frontend.

VFS is frontend-owned file access. Query `VfsInterface`, then use `VfsFile` and
`VfsDirectory` RAII handles. Access modes, seek origins, stat flags, and hints
are typed so call sites do not pass raw VFS bitmasks.

Use normal Rust files only when the core is intentionally bypassing frontend VFS
policy. For frontend-mediated paths, prefer the VFS wrappers.
