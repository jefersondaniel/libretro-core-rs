# Environment and Core Options

`Environment` wraps `RETRO_ENVIRONMENT_*` commands with typed methods. Use it
from `on_set_environment` for setup-time registration and from
`runtime.environment()` when a runtime command is appropriate.

Common setup:

```rust,ignore
fn on_set_environment(&mut self, env: &mut Environment<'_>) {
    let _ = ContentContract::new("bin|dat")
        .with_support_no_game(true)
        .register_environment(env);

    let _ = env.set_variables(&[
        VariableDefinition::new("my_core_mode", "Mode; auto|fast|accurate"),
    ]);
}
```

Prefer `CoreOptions` for modern options. It can register v2 definitions and
generate older fallbacks when a frontend only supports older option APIs.

Environment helpers also cover logging, messages, rotation, overscan,
controller descriptions, input descriptors, AV enable hints, fast-forward
state, throttle state, device power, language, paths, achievements, and
hardware-render negotiation.

Keep command failures meaningful. A frontend returning `false` may mean
"unsupported", "rejected this specific request", or "invalid in this phase"
depending on the command. Do not collapse all failures into one generic state
when the typed API can expose better semantics.

Reference: [Core Options Translation](https://github.com/jefersondaniel/libretro-core-rs/blob/main/spec/core-options-translation.md).
