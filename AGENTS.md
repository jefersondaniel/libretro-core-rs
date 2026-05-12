# AI Agent Guidance

## Project Scope

This repository is a generic Rust workspace for building libretro cores and
OpenGL-backed libretro helpers. Keep examples, tests, comments, package
metadata, and fixture strings reusable and product-neutral.

## API Design Principles

Developer ergonomics is the primary design constraint for this workspace. The
public API should let core authors express libretro and OpenGL intent without
manual FFI ceremony, unsafe blocks, or raw numeric constants at call sites.

Follow these principles when adding or changing APIs:

- Keep public APIs Rust-first even when the underlying ABI is C-first.
- Do not expose `unsafe` in normal core-author workflows. Unsafe code belongs
  behind small, audited wrapper boundaries.
- Do not require callers to pass magic numbers or raw GL/libretro flags when a
  typed enum, newtype, builder, or helper can encode the valid choices.
- Prefer typed enums for OpenGL targets, formats, capabilities, draw modes,
  buffer usages, texture parameters, and libretro context/device choices.
- Use the narrowest applicable enum for an operation so callers cannot pass a
  flag that is meaningless for that context.
- Prefer Rust-native inputs and outputs: `&str`, slices, enums, `Option`,
  `Result`, owned values, and return values instead of raw pointers, mutable
  out-params, `CString`, or `CStr`.
- Preserve upstream contracts exactly, including multi-part callback results,
  pointer lifetime rules, hardware-render lifecycle rules, and context-family
  capability differences.
- Add higher-level helpers only when they remove real repeated setup that most
  cores would otherwise need to copy.
- Keep examples complete and ergonomic. Even small examples should show the
  full practical lifecycle: content contract, AV timing, input polling, audio
  pacing, frame submission, context reset, context destroy, and cleanup.
- Prefer visible, diagnosable failure paths over silent black screens or
  low-information errors.

For APIs that naturally represent combinable flags, consider `enumflags2`
before introducing ad hoc integer bitmasks or loosely typed flag wrappers. Use a
bitflag set only when combinations are valid for the target operation; otherwise
keep separate enums or typed helper methods.

## Commit Messages

Always use Conventional Commits for commit messages.

Allowed examples:

- `feat: add content override helper`
- `fix: preserve system info pointer lifetime`
- `test: cover GLES2 texture cleanup`
- `docs: document workspace layout`
- `refactor: split diagnostic text setup`
- `chore: update package metadata`

Use a scope when it adds clarity, for example:

- `fix(glsym): gate texture arrays on live context version`
- `test(wrapper): reject undersized software frames`

Avoid vague messages such as `update`, `changes`, or `fix stuff`.

## Validation

Before finishing changes, run:

```sh
cargo test --workspace
```
