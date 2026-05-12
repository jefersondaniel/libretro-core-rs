# libretro-core-rs

Rust helpers for building libretro cores.

This workspace contains:

| Crate | Purpose |
|---|---|
| `libretro-core` | Safe-ish Rust wrapper around the libretro ABI, AV/content helpers, hardware-render negotiation, and public `glsym` OpenGL symbol access. |
| `libretro-diagnostics` | Optional diagnostic GL/text/frame helpers for cores that need visible failure output. |
| `examples/software-libretro` | Minimal software framebuffer core. |
| `examples/retrocompat-libretro` | OpenGL/GLES compatibility triangle and text diagnostic core. |
| `examples/demo-libretro` | Generic OpenGL/input/audio demo core. |

Run the test suite with:

```sh
cargo test --workspace
```

## Building Example Cores

All example cores are `cdylib` crates that export the standard `retro_*`
symbols through `libretro-core`.

Example packages:

| Package | Output library | Info file |
|---|---|---|
| `demo-libretro` | `libdemo_libretro.so` | `examples/demo-libretro/demo_libretro.info` |
| `software-libretro` | `libsoftware_libretro.so` | `examples/software-libretro/libsoftware_libretro.info` |
| `retrocompat-libretro` | `libretrocompat_libretro.so` | `examples/retrocompat-libretro/libretrocompat_libretro.info` |

### Linux x86_64 glibc

Install the target if needed:

```sh
rustup target add x86_64-unknown-linux-gnu
```

Build a core:

```sh
cargo build -p demo-libretro --release --target x86_64-unknown-linux-gnu
cargo build -p software-libretro --release --target x86_64-unknown-linux-gnu
cargo build -p retrocompat-libretro --release --target x86_64-unknown-linux-gnu
```

Outputs:

```text
target/x86_64-unknown-linux-gnu/release/libdemo_libretro.so
target/x86_64-unknown-linux-gnu/release/libsoftware_libretro.so
target/x86_64-unknown-linux-gnu/release/libretrocompat_libretro.so
```

### Linux ARMv7-A hard-float glibc 2.17

This target is useful for older ARMv7 Linux frontends that need a conservative
glibc floor. It requires `cargo-zigbuild`.

Install tools:

```sh
rustup target add armv7-unknown-linux-gnueabihf
cargo install cargo-zigbuild
```

Build a core:

```sh
cargo zigbuild -p software-libretro --release --target armv7-unknown-linux-gnueabihf.2.17
cargo zigbuild -p retrocompat-libretro --release --target armv7-unknown-linux-gnueabihf.2.17
```

Outputs:

```text
target/armv7-unknown-linux-gnueabihf/release/libsoftware_libretro.so
target/armv7-unknown-linux-gnueabihf/release/libretrocompat_libretro.so
```

Optional ELF checks:

```sh
readelf --version-info target/armv7-unknown-linux-gnueabihf/release/libretrocompat_libretro.so
readelf -A target/armv7-unknown-linux-gnueabihf/release/libretrocompat_libretro.so
```

The version-info output should not require glibc symbols newer than `GLIBC_2.17`.
The ARM attributes should report ARMv7 hard-float ABI attributes appropriate for
the selected target.

### Linux ARMv8-A glibc

Install tools:

```sh
rustup target add aarch64-unknown-linux-gnu
cargo install cargo-zigbuild
```

Build a core:

```sh
cargo zigbuild -p demo-libretro --release --target aarch64-unknown-linux-gnu
cargo zigbuild -p software-libretro --release --target aarch64-unknown-linux-gnu
cargo zigbuild -p retrocompat-libretro --release --target aarch64-unknown-linux-gnu
```

Outputs:

```text
target/aarch64-unknown-linux-gnu/release/libdemo_libretro.so
target/aarch64-unknown-linux-gnu/release/libsoftware_libretro.so
target/aarch64-unknown-linux-gnu/release/libretrocompat_libretro.so
```

### Linux ARMv8-A musl

For ARM64 musl shared libraries, disable static CRT linkage.

Install tools:

```sh
rustup target add aarch64-unknown-linux-musl
cargo install cargo-zigbuild
```

Build a core:

```sh
RUSTFLAGS="-C target-feature=-crt-static" \
  cargo zigbuild -p demo-libretro --release --target aarch64-unknown-linux-musl
```

Output:

```text
target/aarch64-unknown-linux-musl/release/libdemo_libretro.so
```

### Symbol Checks

The built shared library should export the standard libretro symbols:

```sh
readelf -Ws target/x86_64-unknown-linux-gnu/release/libdemo_libretro.so \
  | rg "retro_(api_version|set_environment|set_video_refresh|set_audio_sample|set_audio_sample_batch|set_input_poll|set_input_state|init|deinit|get_system_info|get_system_av_info|load_game|unload_game|run)"
```

Hardware-render examples should not directly link platform GL libraries; they
load GL entry points through the frontend callback path:

```sh
readelf -d target/x86_64-unknown-linux-gnu/release/libretrocompat_libretro.so \
  | rg "libGL|libEGL|libGLES" || true
```
