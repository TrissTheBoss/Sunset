# Sunset

Sunset is an independent Rust project with a long-term goal of reproducing vanilla Minecraft Java Edition 26.3 behavior, then improving performance. The project is intended to remain free for its lifetime. Vanilla multiplayer compatibility and local import of user-provided Minecraft assets are goals, not current features.

**Current status: framework scaffold only.** This repository is not playable, does not implement Minecraft behavior, and makes no compatibility or performance claims. It contains no Minecraft assets.

Start with the [PRD](docs/PRD.md) and [core philosophy](docs/CORE_PHILOSOPHY.md). The [architecture](docs/ARCHITECTURE.md) describes planned boundaries; the [parity matrix](docs/PARITY_MATRIX.md) tracks evidence and gaps; [agent workflow](docs/AGENT_WORKFLOW.md) documents project work practices; and [release policy](docs/RELEASES.md) governs compatibility and distribution claims. Design decisions are tracked in [ADRs](docs/DECISIONS.md).

## Build the scaffold

Install the Rust stable toolchain; CI and local development use the same stable channel via [`rust-toolchain.toml`](rust-toolchain.toml), then run:

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

The `sunset-foundation` crate contains only project status metadata. No game window or gameplay is available.

Foundation source prereleases are manually dispatched from the reviewed full commit SHA using the `Foundation source prerelease` workflow. They publish source archives only and are labeled non-playable.

## Contributing

Compatibility work must include reproducible reference evidence. Do not add proprietary game assets to the repository.
