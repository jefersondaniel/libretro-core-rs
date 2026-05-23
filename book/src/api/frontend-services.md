# Frontend Services

Frontend services are optional capabilities discovered through environment
queries. The typed API groups them by the contract they represent:

- Logging and messages: `Logger`, `ExtendedMessage`, message targets, progress,
  and kinds.
- Performance: CPU feature flags, counters, frontend time, and tick values.
- Input/output devices: rumble, LED, sensors, location, camera, microphone,
  MIDI, and netpacket interfaces.
- Frontend state: language, username, paths, power state, fast-forward,
  throttling, refresh/sample rates, and audio/video enable flags.

Treat service lookup as capability discovery. Many frontends do not implement
every optional interface, so APIs such as `MicrophoneInterface::available` or
empty interface values let a core degrade intentionally.

Event-like service callbacks should follow the event bus design used by
`CoreEventConfig`: register listeners with `add_*_listener` methods, remove
them with the matching `remove_*_listener` methods when needed, then let the
wrapper install the low-level callback at the correct time. Single-registration
hooks should keep callback wording instead; for example, frame timing uses
`set_frame_time_callback` and `clear_frame_time_callback` because the frontend
receives one reference interval. Query/command services should stay as explicit
typed service handles.
