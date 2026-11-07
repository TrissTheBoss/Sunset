# Architecture

## Status

Sunset is currently a documentation and build-system scaffold. It is not a game, does not launch, and does not claim Java Edition compatibility. No gameplay systems or Minecraft assets are included.

## Product constraints

The project is an independent Rust reimplementation effort. The compatibility target is vanilla Minecraft Java Edition 26.3, with behavioral parity as the first priority and performance as the second. Exact version-specific behavior must be established from reproducible observations and documented evidence; this scaffold intentionally encodes no version constants or compatibility claims.

The intended long-term project is free for its lifetime and includes vanilla multiplayer compatibility. Multiplayer protocol and server behavior are future research and implementation work, not current capabilities.

Users may provide local Minecraft asset files for import. Sunset will not bundle proprietary game assets. Any future importer must keep user-provided files local by default, document the expected source/version and transformation, and avoid redistributing imported assets.

## Planned boundaries

Future code should be divided by stable responsibilities rather than by assumed game features:

- **compatibility evidence**: version-scoped observations, fixtures, and traceable test provenance;
- **simulation**: deterministic tick-based transitions, separate from render cadence and wall-clock time. Numeric edge behavior that can affect parity must be specified and measured against Java behavior before choosing Rust representations.
- **world and integrated server**: world state and server-side simulation should run in-process for single-player, with client/server boundaries designed so a remote vanilla server can later be a distinct mode.
- **world generation**: seed and version scoped, deterministic, independently testable against reference outputs.
- **storage**: world saves and settings with explicit formats, atomic update strategy, and migration policy before persistence is implemented.
- **datapacks and registries**: data-driven content loading with version-scoped registry bootstrap and validation. Registry state must not be accidentally shared across incompatible protocol/version contexts.
- **client presentation**: logical-pixel UI layout and input provenance, plus windowing, audio, and rendering. Rendering API, shader strategy, and antialiasing remain undecided until requirements and platform constraints are documented in an architecture decision record (ADR).
- **assets**: local discovery/import and runtime loading, with archive size limits, path traversal defenses, and no checked-in proprietary payloads.
- **networking**: protocol state machine, login/authentication, registry bootstrap, and play states must be modeled against a named version. Secrets should use the operating system's credential vault where available; never log or store tokens in plaintext. Multiplayer is unimplemented.
- **tools and differential oracle**: inspection and validation utilities, including a future harness for comparing deterministic traces with a pinned vanilla reference build. A result is useful only with reproducible inputs, target build identity, and captured expected/observed output.

These are planning boundaries, not implemented crates. The initial workspace contains only a small foundation crate so CI has a real Rust target without inventing game behavior. Add a boundary as a crate only when there is concrete implementation work for it. Do not select a graphics backend, authentication integration, or persistent world format by speculation; record consequential choices in ADRs.

## Engineering rules

1. Scope claims to an exact Java Edition version and tested behavior. Label unknowns as unknown.
2. Preserve parity evidence: record scenario, inputs, expected/observed result, reference build, and test method.
3. Keep deterministic simulation independent of rendering, audio, and real time.
4. Do not add game assets, extracted assets, or redistributable copies of proprietary data to the repository.
5. Do not call a build a playable release until an actual end-to-end client exists and the release criteria in [RELEASES.md](RELEASES.md) are met.
6. Keep the baseline dependency-free where practical; every dependency needs a concrete use and license review.

## Current implementation

The `sunset-foundation` crate currently exposes only project identity and status metadata. It is not a gameplay API and should remain small until the first evidenced subsystem is approved for implementation.
