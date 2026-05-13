# Memory, Savestates, and Serialization

The memory API maps libretro memory regions and memory maps into typed Rust
values:

- `MemoryRegion` identifies save RAM, RTC, system RAM, video RAM, and unknown
  regions without raw constants at call sites.
- `CoreMemory` returns readonly or readwrite borrowed slices.
- `MemoryMapDescriptor` describes address-space mappings with typed address,
  mask, offset, length, alignment, and access-size values.
- `SoftwareFramebufferRequest` and `SoftwareFramebuffer` model current software
  framebuffer access.

Savestates use the standard `serialize_size`, `serialize`, and `unserialize`
trait methods. Pair them with `SavestateContext` and `SerializationQuirks` when
the frontend needs extra information about determinism, size stability, or
runtime restrictions.

The key ergonomic rule is to keep host pointers and emulated addresses separate.
Use the memory newtypes instead of raw `usize` fields so reviews can tell
whether a number is a host offset, an emulated address, a mask, or a length.
