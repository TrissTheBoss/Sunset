# Requirements work log

## Assignment

Create only `docs/PRD.md`, `docs/PARITY_MATRIX.md`, and this log for the proposed free lifetime, pure-Rust Minecraft Java 26.3-compatible client/integrated experience. Priorities are exact vanilla parity first, performance second. Include standard multiplayer, local asset import (no bundled Minecraft assets), GUI, movement/physics, building, anti-aliasing, shaders, resource packs and settings.

## Decisions recorded

- Minecraft Java Edition 26.3 release identity is verified from the official release article (published 2026-09-15). Artifact hashes, protocol identifiers, and complete behavior inventory remain pending local oracle capture.
- Added verified target facts: resource pack format 97.1, data pack format 121.0, SDL3/scancodes, borderless default, relative mouse behavior, 320x240 minimum window, whitelist default enabled, `/posteffect` custom resource-pack post effects, and an additions inventory covering a biome, plants, cushions and straw beds. Feature semantics remain unverified until compared with the target.
- 100% parity is a release ambition and measurable gate, not a current status or claim.
- The product is original Rust implementation, with no copied proprietary code/assets and no bundled Minecraft assets. User-provided local installation is the asset source.
- Multiplayer requires ordinary legitimate authentication; no auth, entitlement, or online access bypass.
- Matrix evidence is required for a pass; initial matrix rows are all unverified.
- PRD includes client and integrated server/world generation/persistence/protocol/authentication and safety/legal boundaries.

## Actions

- Inspected workspace root; it was empty and `rg` found no `AGENTS.md` guidance.
- Created the three assigned documentation artifacts. No network access, git mutation, builds, or tests were performed.

## Open gaps for review

- Capture exact client/server artifact hashes, protocol identifiers, and the full feature inventory from the local oracle before treating implementation scope as frozen.
- Obtain product-owner/legal review of clean-room research, local-import and distribution model, trademark wording, and auth flow.
- Confirm first supported OS/GPU matrix and measure vanilla baseline before setting numeric performance budgets.
- Decide the intended meaning of "pure Rust" around native graphics/window/audio APIs.
- Decide named shader compatibility subset and account types after target/API research.
- Expanded criteria for mechanics, tick ordering, worldgen numeric semantics, saves, UI, multiplayer social/Realms classification, and separate shader tracks.
- Aligned release parity gate across every applicable baseline priority and matrix severity levels S0-S3.
- Added required Sunset AA enhancement acceptance rows, distinct from vanilla parity: effective backend-appropriate mode, separate settings, HUD/readability and performance checks, and safe unsupported-mode behavior.

