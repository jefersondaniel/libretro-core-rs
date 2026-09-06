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


## [0.3.1](https://github.com/jefersondaniel/libretro-core-rs/compare/libretro-core-v0.3.0...libretro-core-v0.3.1) (2026-09-06)


### Bug Fixes

* **wrapper:** preserve the default hardware framebuffer ([#3](https://github.com/jefersondaniel/libretro-core-rs/issues/3)) ([118f296](https://github.com/jefersondaniel/libretro-core-rs/commit/118f2961ccaeae7430142d0f5f865de968e0670c))

## [0.3.0](https://github.com/jefersondaniel/libretro-core-rs/compare/libretro-core-v0.2.0...libretro-core-v0.3.0) (2026-05-23)


### Features

* add DOM-style event listeners ([d0055a1](https://github.com/jefersondaniel/libretro-core-rs/commit/d0055a179fdd577d850ae4510681c091d06083e3))
* add unified gl facade ([9c0daf6](https://github.com/jefersondaniel/libretro-core-rs/commit/9c0daf614f0738a35e4f9769da0add8b5a78497a))

## [0.2.0](https://github.com/jefersondaniel/libretro-core-rs/compare/libretro-core-v0.1.0...libretro-core-v0.2.0) (2026-05-15)


### Features

* **glsym:** expand typed OpenGL wrappers ([b3af2df](https://github.com/jefersondaniel/libretro-core-rs/commit/b3af2df5bfe6b18c3801a2e2775586652cdc5076))
* **libretro-core:** support armv7 32-bit ABI layout assertions ([0ed9064](https://github.com/jefersondaniel/libretro-core-rs/commit/0ed906475478335c8aece899f21a6f4344739360))
* **libretro:** add typed libretro coverage ([354354a](https://github.com/jefersondaniel/libretro-core-rs/commit/354354aa30521d18da9de85e85db915462b139a0))

## Changelog

All notable changes to `libretro-core` will be documented in this file.
