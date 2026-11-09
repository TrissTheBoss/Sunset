# Product requirements

## Scope and profiles

Build an original memory-safe Minecraft Java Edition client, integrated server and dedicated server. Eventual scope includes survival, creative, adventure, spectator, all dimensions/worldgen/entities/blocks/states/redstone, commands and administrative objects, storage/networking/UI/audio/accessibility and packs. Target Java 26.3 provisionally, based on the existing README. If M0 cannot verify that reference, block and decide explicitly rather than silently switching.

Bedrock and execution of existing Java mod binaries are not initial promises. Multiple Java versions need separate adapters and evidence. First certify the owner's Windows configuration, then Linux/macOS capability tiers; hardware details are currently unknown.

| Profile | Rules |
| --- | --- |
| Vanilla Fidelity | Native resolution; target geometry/light/fog/materials; no visual enhancements |
| Enhanced, proposed default | Identical simulation; distant LOD; Auto upscaling with native fallback |
| Modded | Explicit module graph and isolated saves; changed behavior clearly labeled |

Compatibility Session is a policy layered over either render profile: legitimate account, matching protocol, server authority, no gameplay-changing local modules. Cosmetic rendering does not prove compatibility. Strict visual parity applies only to Fidelity.

## Requirements

| ID | Requirement | Acceptance evidence |
| --- | --- | --- |
| REQ-01 | Safe Rust runtime, custom WGSL, no first-party C/C++ | Unsafe/dependency audit and owned renderer |
| REQ-02 | Complete observable target parity | Inventory mapping and mandatory differential evidence; no known unresolved critical mismatch |
| REQ-03 | 4,000+ FPS stretch on owner machine | Repeated matched BENCH-01 at native resolution |
| REQ-04 | Smooth performance | Frame/tick tails, streaming and memory gates |
| REQ-05 | Foundation before mass content | M3 exit |
| REQ-06 | Culling and greedy meshing | Conservative visibility and reference-mesh equivalence |
| REQ-07 | Attractive fast LOD | Seams, silhouettes, transitions, invalidation and budgets |
| REQ-08 | Standard upscaling | Auto/Off/scales, native HUD, quality tests |
| REQ-09 | Every game domain extensible | Surface inventory and independent example modules |
| REQ-10 | Own mod platform | Versioned contracts, bounds and cheap no-hook paths |
| REQ-11 | Vanilla-server interoperability | Authorized protocol/state/authentication matrix |
| REQ-12 | Legal local resources | Read-only importer/provenance and clean distribution |
| REQ-13 | Resource/data packs | Pinned format, semantics, precedence and reload tests |
| REQ-14 | Shader packs | Native WGSL plus separately verified Iris/OptiFine-format compatibility |
| REQ-15 | Commands/admin content | Grammar, permissions, completion, command blocks, structures and full target inventory |
| REQ-16 | GitHub/Wiki | Recoverable replacement, release history and published versioned documentation |
| REQ-17 | Clean production source | No first-party production comments/dead code; licenses preserved |
| REQ-18 | Unattended verification | Deterministic runner with meaningful blocked/failure statuses |
| REQ-19 | Vanilla-style settings | Familiar navigation, local UI assets and engine controls |
| REQ-20 | Save integrity | Compatible semantics, backups, atomic writes, migration/recovery |
| REQ-21 | Security/privacy | Bounded parsers, sandboxed modules, secure tokens, no telemetry default |
| REQ-22 | Accessibility/localization | Scaling, rebinding, subtitles, fonts/languages and narration |
| REQ-23 | Resilience | Device/network loss, low memory, missing assets and failed reload |
| REQ-24 | Reproducible distribution | Pinned build, SBOM/licenses/checksums and rollback |

Full scope includes obscure states, scoreboard/command return values, world border, random streams, upgrades and chat/security behavior—not just movement/building. Preview releases list supported features; they do not complete REQ-02. Publish raw verified/required counts and gaps. Use “no known differences in declared scope” with evidence rather than asserting proof of 100%.

Added improvements: private secret-stripped reproduction bundles, per-mod performance attribution, crash-safe saves, provenance tracking, accessible diagnostics, and quality-preserving fallback paths. Each requires tests.

No gameplay simplification for FPS, anticheat evasion, authentication bypass, proprietary redistribution or adopted complete WGSL renderer. Reuse audited Rust cryptography/compression/platform primitives. No general scripting runtime in per-voxel hot paths.
