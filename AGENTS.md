# AI Agent Guidance

## Project Scope

This repository is a generic Rust workspace for building libretro cores and
OpenGL-backed libretro helpers. Keep examples, tests, comments, package
metadata, and fixture strings reusable and product-neutral.

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
