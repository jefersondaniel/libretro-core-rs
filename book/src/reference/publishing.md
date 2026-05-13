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
