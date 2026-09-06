# Changelog

## 1.0.0 (unreleased)

### Breaking changes

- Replace the custom typed GL API and generated dispatch tables with standard
  glow 0.17, re-exported by libretro-core's default `glow` feature.
- Load contexts with `Runtime::create_glow_context()` inside context reset.
  Glow commands retain their standard unsafe contracts; frontend context ownership
  and resource cleanup remain explicit.
- Remove partial-symbol staged GL initialization and public fake GL helpers.
- Migrate text diagnostics to glow, with a single context argument and consuming
  explicit destruction. CPU font and software diagnostic helpers remain.
- Rewrite the OpenGL tutorial, API reference and examples; add a 1.0 migration guide.


All notable changes to `libretro-diagnostics` will be documented in this file.
