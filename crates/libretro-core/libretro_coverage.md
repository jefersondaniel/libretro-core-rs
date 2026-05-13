# libretro.h Coverage Tracker

This file tracks ergonomic coverage of `include/libretro.h`. The source of truth for counts is
`scripts/audit_libretro_coverage.py`; update this file whenever coverage status
or category ownership changes.

<!-- libretro-coverage-counts
kind.constant=236
kind.enum=16
kind.function=25
kind.struct=56
kind.typedef=91
status.typed-wrapper=424
total=424
-->

## Current Snapshot

- Required `retro_*` export functions are covered by the `Core` trait and
  `export_core!`.
- The ergonomic public API currently covers lifecycle basics, system/AV info,
  content contract, simple variables, messages, logging, typed polling for
  common input devices, typed input device subclass IDs, LED and rumble output
  control, keyboard event callbacks, input descriptors,
  frontend path/user/language/JIT queries, rotation, overscan,
  frame-dupe, shutdown, system AV reconfiguration, device power query,
  input capability/max-user/bitmask queries,
  runtime audio/video enable hints, target refresh/sample-rate queries,
  fast-forward state and override hints, throttle state, achievement support,
  minimum audio latency and performance-level hints, savestate context, serialization quirk hints,
  extended frontend messages, software/video/audio callbacks, audio buffer
  occupancy callbacks, frame-time callbacks, async audio callbacks, CPU feature
  flags, performance counters, perf timing, and OpenGL hardware rendering, plus
  current software framebuffer requests and typed framebuffer submission,
  sensor, camera, and location interface helpers, subsystem and disk-control
  descriptors, microphone input, MIDI I/O, netpacket session/callback helpers,
  hardware render interface/context-negotiation helpers, core-options
  registration/update helpers, and VFS file/directory operations.
- Every currently tracked `libretro.h` item has an ergonomic typed wrapper or
  is an internal ABI detail behind one.
- ABI-shaped exports remain behind the `export_core!` and `__private`
  boundaries; normal core-author workflows use typed wrappers.

## Categories

| Category | Planned module | Status | libretro.h names tracked | API shape | Tests and gaps |
| --- | --- | --- | --- | --- | --- |
| `core-abi` | `lib.rs`, `raw.rs`, later `abi.rs` | typed-wrapper | `retro_set_environment`, `retro_init`, `retro_run`, `retro_load_game`, `retro_get_memory_data`, all required exports | Keep C exports macro-backed; move raw trampoline details behind focused ABI modules | Add direct tests for default trait paths not yet exercised |
| `environment` | `environment.rs` | complete in current manifest | Cross-cutting `RETRO_ENVIRONMENT_*` command dispatch, aliases, experimental/private flag bits | Typed methods grouped by domain; shared command wrapper preserves exact integers and optional failure | Support matrix behavior is covered by domain tests; add broader false-return examples only when they clarify user-facing behavior |
| `input` | `input.rs`, `midi.rs` | complete in current manifest | `RETRO_DEVICE_*`, joypad IDs, mouse, keyboard, analog, lightgun, pointer, descriptors, controller info, rumble, LED, MIDI input/output callbacks | `InputPort`, `ControllerDevice`, `ControllerDeviceSubclass`, `ControllerInfo`, `ControllerDescription`, `KeyboardKey`, `KeyboardModifiers`, `KeyboardEvent`, `InputDescriptor`, `InputDescriptorIndex`, `InputDescriptorId`, `LedInterface`, `RumbleInterface`, `SensorInterface`, `MidiInterface`, narrow enums per polled device, typed pointer indexes, joypad bitmask helper, and `InputDeviceCapabilities` flags | Raw polling, controller-device callbacks, controller info forwarding, keyboard callback dispatch, retained input descriptors, subclass IDs, max users, bitmask support, LED/rumble/sensor state forwarding/null callback, MIDI availability/read/write/flush, and device capabilities are tested |
| `memory` | `memory.rs` | complete in current manifest | `RETRO_MEMORY_*`, `RETRO_MEMDESC_*`, `RETRO_MEMORY_ACCESS_*`, `RETRO_MEMORY_TYPE_*`, `RETRO_SERIALIZATION_QUIRK_*`, `retro_savestate_context`, `retro_memory_descriptor`, `retro_memory_map`, `retro_framebuffer`, `retro_game_info_ext`, `retro_subsystem_memory_info`, camera raw framebuffer callback type | `MemoryRegion` removes raw IDs from trait methods; `MemoryMapDescriptor` uses borrowed host slices for accessible ranges, typed address/mask/len/offset newtypes, owned addrspace conversion, `MemoryDescriptorFlags`, exclusive alignment/min-size enums, `SoftwareFramebufferRequest`, framebuffer byte views, `ExtendedGameInfo` borrowed views, framebuffer memory flags, `SavestateContext`, `SerializationQuirks`, `SubsystemMemoryInfo`, and borrowed `CameraRawFrame` cover value-level memory hints | Raw ABI conversion, memory map forwarding, null inaccessible ranges, software framebuffer request/submission, extended game-info queries, subsystem descriptor forwarding, camera raw-frame dispatch, flag encoding, savestate context, and serialization quirk negotiation are tested |
| `options` | `options.rs` | complete in current manifest | `retro_variable`, `retro_core_option_*`, `RETRO_ENVIRONMENT_SET_CORE_OPTIONS*`, display/update callbacks | `CoreOptions`, `CoreOptionDefinition`, `CoreOptionCategory`, `CoreOptionValue`, `CoreOptionDisplay`, and `CoreOptionsVersion` provide v2-first builders, v1/v0 fallback generation, intl entry points, single-option updates, and owned backing storage without raw tables at call sites | Simple variables, version query fallback, v2 categories/labels, v1/v0 fallback generation, intl entry points, display visibility updates, set-variable, callback registration/dispatch, value-count validation, and layout checks are tested |
| `subsystems-disks` | `subsystem.rs`, `disk.rs` | complete in current manifest | `retro_subsystem_info`, ROM/memory info, disk control and ext callback tables | `SubsystemInfo`, `SubsystemRomInfo`, `SubsystemMemoryInfo`, and `SubsystemId` provide retained nested descriptor storage and avoid raw IDs in `load_game_special`; `DiskTrayState`, `DiskIndex`, and `DiskControlInterfaceVersion` hide tray/index/version integers while callback trampolines preserve eject/index/path/label contracts | Subsystem nested descriptor forwarding/storage retention, disk interface version query, legacy and extended disk callback registration/clear, callback dispatch, string copying, and layout checks are tested; still add richer special-load examples when examples cover subsystems |
| `callbacks` | `callbacks.rs`, `perf.rs`, `camera.rs`, `netplay.rs` | complete in current manifest | Frontend callback typedefs, audio callback, frame time, buffer status, proc-address, camera/location/netpacket/perf callbacks | Small registration types with required/optional callback distinction and thread-safety bounds; `AudioCallbackState` avoids raw bool state, `AudioBufferStatus` preserves invalid frontend occupancy values, `FrameTime` preserves signed microseconds, `CoreProcAddress` isolates type-erased extension pointers, `CameraRequest`/frame types hide raw camera callbacks, and `NetpacketSession` bounds frontend send/poll function lifetime to the start/stop contract | Audio callback probe/register/clear, audio buffer status, frame-time registration/clear, proc-address dispatch, camera and location lifecycle/frame dispatch, netpacket registration/send/receive/session dispatch, and perf callback table dispatch are tested |
| `frontend-services` | `environment.rs`, `midi.rs`, `microphone.rs` | complete in current manifest | Directories, username, language, rotation, overscan, frame duping, shutdown, shared-context hint, system AV info, achievements, fast-forwarding, throttle, JIT, device power, MIDI, microphone, message/log service interfaces | Owned strings for frontend-owned path/user values, typed `VideoRotation`, optional capability results, typed `DevicePower`, typed throttle state, typed AV geometry/timing structs, `FastForwardingOverride` with finite ratio helpers, typed controller info registration, proc-address callback registration, typed achievement hint, typed extended-message builder, shared-context request helper, `MidiInterface` with typed delta times, and RAII `Microphone` handles with typed parameters | Path/user/language/JIT/power/fast-forward/throttle, fast-forward override support/set, controller info forwarding, proc-address lookup dispatch, rotation/dupe/overscan/shutdown/shared-context/system AV, achievements, MIDI probe/read/write/flush, microphone open/params/state/read/close, and extended-message APIs are tested |
| `hardware` | `hardware.rs`, OpenGL modules | complete in current manifest | `retro_hw_render_callback`, context type, proc lookup, render interfaces, context negotiation, software framebuffer, shared context | OpenGL-safe wrappers remain typed; `HwRenderInterfaceType`, `HwRenderInterface`, `HwRenderContextNegotiationInterfaceType`, and `HwRenderContextNegotiationInterface` cover base render-interface discovery and context-negotiation versioning without raw command IDs; current software framebuffer uses typed request and exact-field submission | HW render setup/candidate fallback, injected runtime callbacks, current software framebuffer paths, shared-context request, and base render-interface/context-negotiation calls are tested; API-specific Vulkan/D3D extension structs are outside `libretro.h` |
| `vfs` | `vfs.rs` | complete in current manifest | `retro_vfs_*`, `RETRO_VFS_*`, opaque file/dir handles | `VfsInterface`, `VfsInterfaceVersion`, RAII `VfsFile`/`VfsDirectory` handles, typed access modes/hints/seek positions/stat flags, and Rust slices/strings for file and directory operations | Version negotiation, open/read/write/seek/tell/size/truncate/flush/close, remove/rename/mkdir/stat, directory iteration/name/type/close, flag encoding, and layout checks are tested |
| `sensors-camera-location` | `sensors.rs`, `camera.rs`, `location.rs`, `microphone.rs` | complete in current manifest | Sensor action/input IDs, camera callbacks, location callbacks, microphone interface | `SensorInterface`, `Sensor`, `SensorAction`, `SensorRateHz`, `SensorInput`, `CameraInterface`, `CameraCapabilities`, `CameraRequest`, camera frame types, `LocationInterface`, location interval newtypes, `LocationPosition`, `MicrophoneInterface`, `MicrophoneParams`, and RAII `Microphone` handles hide raw action/value IDs, callback out-params, opaque mic pointers, and camera callback tables | Sensor interface lookup, enable/disable false returns, input reads, empty-interface behavior, camera start/stop/frame/lifecycle dispatch, location start/stop/interval/position/lifecycle dispatch, microphone lifecycle/read paths, and layout checks are tested |
| `audio-timing` | `callbacks.rs`, `runtime.rs`, `perf.rs` | complete in current manifest | Audio callback, frame-time callback, buffer status, latency, target refresh/sample rate, AV enable flags, perf time query | `AvEnableFlags`, refresh/sample-rate newtypes, latency newtype, frame-time microseconds, audio callback state, perf microsecond time, and buffer occupancy values that preserve frontend contract details | Target rates, fast-forward, AV flags, latency hints, audio callback, audio buffer status callback, frame-time callback, and perf time query paths are tested |
| `netplay` | `netplay.rs` | complete in current manifest | `retro_netpacket_*`, packet flags, broadcast client, client index | `NetplayClientId`, `NetpacketTarget`, `NetpacketFlags`, `Netpacket`, and `NetpacketSession` encode valid packet delivery modes, client IDs, send targets, received byte slices, and callback-scoped frontend send/poll handles | Client-index query, flag encoding, interface registration/clear, protocol string retention, callback dispatch, send/flush forwarding, optional poll-receive handling, and layout checks are tested |
| `diagnostics-testing` | `perf.rs`, diagnostics crate | complete in current manifest | `retro_log_callback`, `retro_message_ext`, `RETRO_SIMD_*`, perf counters, performance level | Typed logger/messages, `CpuFeatures` bitflags, `PerfInterface`, pinned `PerfCounter`, `PerfTick`, and `PerfTimeMicros` keep retained counter storage and identifiers stable | Logger, extended messages, CPU feature flags, performance-level hints, perf interface lookup, pinned counter registration/start/stop, log, CPU feature, timer, and layout paths are tested |

## ABI And Lifetime Hazards

- Keep every mapped C struct `#[repr(C)]` with pointer-width layout assertions.
- Preserve C calling convention and function pointer optionality.
- Treat C strings and arrays as owned backing storage when frontends may retain
  pointers; document shorter callback-only lifetimes explicitly.
- Distinguish nullable support probes from commands where null data is undefined.
- Preserve stateful multi-part contracts: `need_fullpath` vs content buffers,
  serialize-size stability, disk eject/index order, VFS version negotiation, core
  options fallback, and hardware context reset/destroy order.
- Do not collapse all frontend `false` results into one error class; each
  command needs its own capability/failure semantics.

## Next Slices

1. Move remaining environment and runtime methods out of `lib.rs` without API
   churn, keeping tests green.
2. Add richer examples only where they show full lifecycle value.

## Verification

Run:

```sh
python3 scripts/audit_libretro_coverage.py --check
cargo test --workspace
```

The audit currently checks inventory counts and required category entries. It is
not a replacement for API review, ABI layout assertions, examples, or tests.
