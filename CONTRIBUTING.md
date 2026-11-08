# Contributing & Development Process

This file records the rules for keeping the Sunset repository, its tracking
records, and its GitHub wiki in sync as the project moves through milestones.

## Milestone commit rule

Sunset is built in milestones (`M0`, `M1`, `M2`, ...) as defined in `PRD.md`.
After completing a milestone — and **before** starting the next — commit and
push the whole result:

1. Finish the milestone's player-visible result and its acceptance checks
   (see the milestone's acceptance list in `PRD.md`).
2. Update the tracking records in the same commit:
   - `PROGRESS.md` — mark the checkpoint complete and name the next increment.
   - `PARITY_MATRIX.md` — add/refresh case status and evidence.
   - `CHANGELOG.md` — add an entry for the milestone.
3. Commit with a `M<n>:` prefix, e.g. `M0: first useful pixels`.
4. Push to `main`.

Do **not** push intermediate, broken, or half-finished states. Each pushed
commit should leave the workspace buildable and runnable (PRD section 1), so
the previous playable result always remains available.

## Documentation

- `README.md` — entry point: prerequisites, launch command, controls, limits.
- `PROGRESS.md`, `PARITY_MATRIX.md`, `CHANGELOG.md` — living tracking records.
- The GitHub **wiki** mirrors the documentation in a more readable, cross-linked
  form and is updated alongside each milestone (start at the wiki `Home` page).

## Build & checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run -p sunset -- --dev-fixture building-yard
```

The `rust-toolchain.toml` pins the stable channel plus `rustfmt` and `clippy`.
See `README.md` → "Building on this machine" for the compiler stack/memory
workaround required on the current development machine.
