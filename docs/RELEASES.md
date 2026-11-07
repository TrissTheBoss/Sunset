# Release policy

## Current state

There are no playable releases. The current repository is a framework scaffold and must not be described as a Minecraft clone, compatible client, or multiplayer implementation.

The manually dispatched `Foundation source prerelease` workflow is limited to `v0.1.0-dev.N` tags. It requires an explicit commit SHA reachable from `main`, checks that the tag matches the foundation crate version, and rejects an existing tag that points to another commit. It reruns formatting/lint/tests and publishes a clearly labeled prerelease with a source-only archive. It does not build or attach a game binary. These artifacts are development snapshots, not compatibility releases.

## Versioning and claims

Use semantic versioning for project artifacts once there is a user-facing artifact. Until then, changes may be tracked in source control without assigning compatibility releases. A release must identify its exact intended Java Edition target and clearly distinguish implemented, tested, partial, and unknown behavior.

Never imply full parity from a passing test suite. Compatibility statements must link to a maintained evidence matrix and state its coverage and known gaps. Performance claims must name the hardware, workload, settings, and measurement method.

## Minimum release gates

Before any playable client release:

- the client builds reproducibly from documented source and starts on supported platforms;
- the release's feature and compatibility claims are backed by repeatable tests against the named vanilla reference;
- known gaps, unsupported content, and platform limits are listed;
- the asset-import path is documented and does not redistribute proprietary assets;
- privacy, networking, and multiplayer behavior are documented for the shipped build;
- CI formatting, lint, and automated tests pass, with release artifacts traceable to a source revision.

Before claiming vanilla multiplayer compatibility, test the released client against the named vanilla server version and document supported protocol behavior, authentication assumptions, and limitations. Do not use “multiplayer compatible” as a roadmap label for an unimplemented feature.

## Distribution and assets

The project is intended to remain free for its lifetime. Do not introduce paid tiers, in-app purchases, or monetized access. Do not commit Minecraft binaries, textures, sounds, fonts, or other proprietary assets. Import must operate on user-supplied local files; publish only original code and assets that the project has rights to distribute.
