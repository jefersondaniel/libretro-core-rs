# Publishing and Validation

The book is a standard mdBook:

```sh
mdbook build book
```

Do not commit generated `book/book/` output. CI builds it and publishes the
artifact.

The documentation gate should run:

```sh
cargo test --workspace
cargo doc --workspace --no-deps
mdbook build book
```

Publishing is handled by GitHub Actions on pushes to `main`. Pull requests build
the same source without deploying, so broken chapters, missing mdBook files, and
Rustdoc failures are visible before merge.

## 1.0 glow validation

The breaking release replaces the custom GL API with re-exported glow 0.17.
`release-please-config.json` requests 1.0.0; remove its one-shot `release-as`
override after that release. The diagnostics companion also moves to 1.0.0.
Merging the implementation PR does not itself constitute a crates.io release;
review the generated release PR and its migration notes.

```sh
cargo test --workspace
cargo test -p libretro-core --no-default-features
cargo fmt --all --check
cargo doc --workspace --no-deps
mdbook build book
cargo build --workspace
python3 scripts/smoke_glow.py --core target/debug/libglow_libretro.so --gles 2
python3 scripts/smoke_glow.py --core target/debug/libretrocompat_libretro.so --gles 2
python3 scripts/smoke_glow.py --core target/debug/libdemo_libretro.so --gles 3
```

The smoke harness uses installed Mesa EGL/GLES and Python's standard library;
it downloads nothing. It exercises the examples' 60 Hz / 48 kHz contract,
framebuffer zero/nonzero, real rendered pixels and two context recreation paths.
It is a driver/ABI test, not a RetroArch run. Validate the examples separately in
an installed RetroArch frontend. ARM compile checks are not ARM runtime evidence.
