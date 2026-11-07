# Framework work log

## Scope

Owned files: README, architecture and release documentation, GitHub workflow/templates, ignore rules, Rust workspace/toolchain manifests, foundation crate, and this log. This work provides build and governance scaffolding only.

## Decisions

- Drafted architecture and release policy before creating Rust code.
- Kept the workspace dependency-free and limited implementation to project identity metadata.
- Use the installed Rust stable channel. The exact local compiler observed during setup was `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; the repository does not pin that patch because the toolchain was not installed under that exact channel name and rustup attempted a network sync. CI also tracks stable.
- Treat Java Edition 26.3 parity, performance, local asset import, and vanilla multiplayer as goals; this scaffold does not implement them.
- Keep all proprietary asset data out of source control; ignore conventional local import directories.

## Actions

- Added workspace manifests, formatting/lint/test CI, PR checklist, and bug report template.
- Added an Unreleased changelog entry and a manually dispatched `v0.1.0-dev.N` source-only prerelease workflow. It requires a reviewed full commit SHA, checks crate/tag version agreement, reruns validation, and explicitly labels the artifact non-playable; it emits no compiled game binary.
- Set workspace license metadata to match the repository's MIT license. Tightened prerelease provenance: the selected SHA must be reachable from `main`, and an existing requested tag is accepted only if it already points to that same commit.
- Added architecture boundaries for deterministic simulation, integrated server, worldgen, storage, datapacks/registries, UI/rendering, import security, protocol/auth, and future differential testing. These remain design notes pending implementation and evidence.
- Removed a metadata-only test that duplicated constants; CI compilation and lint checks are the meaningful scaffold validation.

## Validation

Using installed `+stable` (`rustc 1.99.1`), generated `Cargo.lock`, formatted, and passed:

- `cargo +stable test --workspace --locked` (0 tests; crate compiles)
- `cargo +stable clippy --workspace --all-targets --locked -- -D warnings`
- `cargo +stable fmt --all -- --check`

No gameplay or compatibility tests exist.

The new release workflow was reviewed structurally but was not dispatched; no release was published.

## Git setup limitation

Initialized the repository on `main`. Adding the requested GitHub remote was blocked because this sandbox cannot write `.git/config` (permission denied); the current account also differs from the workspace owner, so Git reports dubious ownership. No global `safe.directory` exception was added. Root/maintainer can set the remote after the workspace permissions are corrected.
