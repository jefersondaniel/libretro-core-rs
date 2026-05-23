# Memory, Savestates, and Serialization

This chapter covers four related libretro surfaces:

- **Memory regions** — exposing save RAM, system RAM, VRAM to the frontend.
- **Memory maps** — describing the emulated address space so debuggers and
  cheat engines can attach.
- **Savestates** — serializing and restoring core state.
- **Software framebuffers** — writing directly into a frontend-provided
  pixel buffer.

A key design rule in this crate is that *host pointers and emulated
addresses are never the same type*. Use the newtypes
(`EmulatedAddress`, `MemoryMapOffset`, `MemoryMapMask`, `MemoryMapLen`) so
that reviewers can tell at a glance whether a number is a host offset, an
emulated address, a mask, or a length.

## Exposing Memory Regions

Implement `memory_region` on the `Core` trait when a frontend should be
able to read save RAM, VRAM, or system RAM (for save files, debuggers,
achievements, or cheat overlays):

```rust,ignore
fn memory_region(&mut self, region: MemoryRegion) -> Option<CoreMemory<'_>> {
    match region {
        MemoryRegion::SaveRam => Some(CoreMemory::read_write(&mut self.save_ram)),
        MemoryRegion::SystemRam => Some(CoreMemory::read_only(&self.work_ram)),
        MemoryRegion::VideoRam => Some(CoreMemory::read_only(&self.vram)),
        _ => None,
    }
}
```

`MemoryRegion` covers `SaveRam`, `Rtc`, `SystemRam`, `VideoRam`, `Rom`, and
`Unknown(u32)`. `CoreMemory::read_only` and `read_write` wrap the borrowed
slice and tell the frontend which access modes are allowed.

Return `None` from `memory_region` for any region the core does not
expose. Do not return a zero-length slice — the frontend reads `None` as
"unsupported", which is the right behavior for unmapped regions.

## Memory Map Descriptors

A memory map tells the frontend how regions of host memory line up against
the emulated address space. Register one with `Environment::set_memory_maps`
during setup:

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    let work_ram = MemoryMapDescriptor::from_slice(
        Some("WRAM".to_string()),
        EmulatedAddress::from(0xC000),
        &mut self.work_ram,
    )
    .with_flags(MemoryDescriptorFlag::SystemRam.into())
    .with_select(MemoryMapMask::from(0xE000))
    .with_len(MemoryMapLen::from(0x2000));

    let cartridge = MemoryMapDescriptor::new_inaccessible(
        Some("ROM".to_string()),
        EmulatedAddress::from(0x0000),
        MemoryMapMask::from(0x8000),
    );

    let _ = env.set_memory_maps(&[work_ram, cartridge]);
}
```

Two constructor shapes:

- `from_slice(addrspace, start, &mut [u8])` for a region with a real backing
  buffer. The pointer is borrowed for the lifetime of the descriptor.
- `new_inaccessible(addrspace, start, select)` for regions the core does not
  expose to debuggers (open bus, ROM, IO ports).

Builder methods on `MemoryMapDescriptor`:

| Method | Purpose |
| --- | --- |
| `with_flags(MemoryDescriptorFlags)` | `Constant`, `BigEndian`, `SystemRam`, `SaveRam`, `VideoRam`. |
| `with_alignment(MemoryDescriptorAlignment)` | `TwoBytes`, `FourBytes`, `EightBytes`. |
| `with_min_access_size(MemoryDescriptorMinAccessSize)` | Smallest legal access width. |
| `with_offset(MemoryMapOffset)` | Host-side offset into the backing buffer. |
| `with_select(MemoryMapMask)` | Select mask in emulated address space. |
| `with_disconnect(MemoryMapMask)` | Address lines that disconnect. |
| `with_len(MemoryMapLen)` | Length of the mapped region in bytes. |

The wrapper retains all descriptor strings internally, so the slice you
pass to `set_memory_maps` can be a borrowed temporary.

## Savestates

Three `Core` trait methods cover savestate serialization:

```rust,ignore
fn serialize_size(&self) -> usize {
    self.core_state_size_bytes()
}

fn serialize(&self, data: &mut [u8]) -> bool {
    self.write_state_into(data).is_ok()
}

fn unserialize(&mut self, data: &[u8]) -> bool {
    self.read_state_from(data).is_ok()
}
```

`serialize_size` is called first; the frontend allocates that many bytes
and then calls `serialize`. On restore, `unserialize` receives the bytes
written previously. Both must return `true` on success and `false` on any
failure that should be surfaced to the user.

`serialize_size` should be deterministic for a given core configuration.
If it can change at runtime (variable-size cores), declare that explicitly
through serialization quirks (next section).

## Savestate Context and Quirks

Savestates serve four different scenarios in libretro frontends, and not
all of them want identical behavior. `Environment::savestate_context()`
tells the core which scenario is active:

```rust,ignore
let mut env = runtime.environment();
match env.savestate_context() {
    Some(SavestateContext::RunaheadSameInstance) => self.fast_serialize(),
    Some(SavestateContext::RollbackNetplay) => self.netplay_serialize(),
    _ => self.standard_serialize(),
}
```

`SavestateContext` variants: `Normal`, `RunaheadSameInstance`,
`RunaheadSameBinary`, `RollbackNetplay`, `Unknown(i32)`.

If the core's serialization has limitations, declare them so the frontend
can avoid scenarios that would corrupt:

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    let quirks = SerializationQuirk::SingleSession
        | SerializationQuirk::EndianDependent;
    let _ = env.set_serialization_quirks(quirks);
}
```

`SerializationQuirk` flags: `Incomplete`, `MustInitialize`,
`CoreVariableSize`, `FrontendVariableSize`, `SingleSession`,
`EndianDependent`, `PlatformDependent`.

The return value of `set_serialization_quirks` is the subset of quirks the
frontend accepted — useful when negotiating with older frontends.

## Cheats

Two optional `Core` trait methods cover cheat code application:

```rust,ignore
fn cheat_reset(&mut self) {
    self.active_cheats.clear();
}

fn cheat_set(&mut self, index: CheatIndex, enabled: bool, code: Option<CheatCode<'_>>) {
    let Some(code) = code else { return; };
    if enabled {
        let text = code.to_string_lossy();
        self.active_cheats.insert(index.get(), text.into_owned());
    } else {
        self.active_cheats.remove(&index.get());
    }
}
```

`CheatIndex::new(u32)` and `CheatIndex::get()` move between the wrapper and
raw indices. `CheatCode<'_>` borrows a NUL-terminated string from the
frontend; use `as_c_str`, `to_str`, or `to_string_lossy` to consume it.

## Software Framebuffer

`SoftwareFramebuffer` is a frontend-provided pixel buffer that lets a
software core write directly into frontend memory instead of submitting a
copy. Request access through `Environment::current_software_framebuffer`:

```rust,ignore
fn run(&mut self, runtime: &mut Runtime<'_>) {
    let request = SoftwareFramebufferRequest::new(WIDTH, HEIGHT)
        .with_access(FramebufferMemoryAccess::Write.into());

    let mut env = runtime.environment();
    if let Some(mut fb) = env.current_software_framebuffer(request) {
        let pitch = fb.pitch();
        if let Some(bytes) = fb.bytes_mut() {
            paint_frame(bytes, pitch);
        }
        let _ = runtime.video_refresh_frame(&self.cached_pixels, WIDTH, HEIGHT, pitch);
    } else {
        let _ = runtime.video_refresh_frame_with_audio(
            &self.framebuffer, WIDTH, HEIGHT, self.pitch, &self.audio,
        );
    }
}
```

Useful `SoftwareFramebuffer` methods:

- `width()`, `height()`, `pitch()`, `format()`.
- `access()` — what the frontend granted (may be narrower than requested).
- `memory()` — cache flags such as `Cached`.
- `bytes()` returns `Some(&[u8])` only if read access is granted.
- `bytes_mut()` returns `Some(&mut [u8])` only if write access is granted.

Always handle the `None` fallback path: not every frontend exposes a
software framebuffer, and the answer can change between frames.

## Reference

- [Runtime Video and Audio](runtime-video-audio.md) — normal frame
  submission helpers.
- [Frontend Services](frontend-services.md) — additional services such as
  rumble and LED that may interact with save data.
