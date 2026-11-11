# Sunset engineering handbook
Revision 2 · 10 November 2026 · Rust + WGSL · DeepSeek V4 Pro · VS Code + Cline

This is a specification and management kit, not implemented game code. The handbook branch replaces the old active file tree with these documents while retaining its ancestry. Main and the live Wiki are not changed by this delivery. GitHub's Download ZIP for this branch provides the complete bundle.

Start with [PRD](docs/01-PRD.md), [parity](docs/02-Parity.md), [roadmap](docs/10-Roadmap.md), [backlog](docs/tracking/BACKLOG.md), [Cline setup](docs/11-Cline.md), then [first task](docs/12-First-Task.md). Follow [repository replacement](docs/13-GitHub-Wiki.md) before making this the active project. Keep [progress](docs/tracking/PROGRESS.md), [coverage](docs/tracking/PARITY.md), [risks](docs/tracking/RISKS.md), [environment](docs/tracking/ENVIRONMENT.md) and [decisions](docs/decisions/INDEX.md) current.

The existing Sunset README inspected on 9 November 2026 identifies Java Edition 26.3 and a small raw-wgpu prototype. Keep 26.3 as the proposed reference, subject to M0 verification. Do not inherit its implementation or completion claims. Exact artifacts, protocol and pack versions must be pinned before parity work.

## Important corrections

- **3,000 FPS is a MUST** on the named workload BENCH-01 (Fidelity, native resolution, owner machine): 333 microseconds per frame, gated from M2 onward ([ADR-0002](docs/decisions/ADR-0002-performance-target.md)). 4,000 FPS (250 µs) stays a stretch goal. Neither is a promise for every scene.
- Full observable parity is the objective; finite tests establish coverage, not a mathematical proof of all behavior.
- LOD and upscaling intentionally alter images. Enhanced is separate from native-resolution Vanilla Fidelity.
- Normal protocol/gameplay behavior supports server interoperability. Acceptance by every third-party anticheat cannot be guaranteed.
- Rust and WGSL fit the project. C/C++ do not inherently require garbage collection; leaks and GC are different concerns. Safe Rust still permits logical leaks, exhaustion and deadlocks.
- Comments ordinarily are not executable instructions. The requested no-comment rule applies to first-party production Rust/WGSL source; design explanations live in Markdown. Required licenses remain.
- Foundation first means renderer, automation, extension contracts and performance infrastructure tested using representative synthetic scenes before mass content. Do not implement every imaginable optimization before measuring real work.

## Contents

| Document | Purpose |
| --- | --- |
| [01 PRD](docs/01-PRD.md) | Requirements and scope |
| [02 Parity](docs/02-Parity.md) | Reference lock and evidence |
| [03 Architecture](docs/03-Architecture.md) | Language and ownership boundaries |
| [04 Renderer](docs/04-Renderer.md) | Custom pipeline, LOD, upscaling, settings |
| [05 Optimizations](docs/05-Optimizations.md) | Candidates with correctness gates |
| [06 Modding](docs/06-Modding-Packs.md) | Modules and pack compatibility |
| [07 Testing](docs/07-Automated-Testing.md) | Unattended runner contract |
| [08 Visual QA](docs/08-Visual-QA.md) | Textures/models/screenshots |
| [09 Benchmarks](docs/09-Benchmarks.md) | Repeatable performance evidence |
| [10 Roadmap](docs/10-Roadmap.md) | Foundation-first milestones |
| [11 Cline](docs/11-Cline.md) | Model configuration and workflow |
| [12 First task](docs/12-First-Task.md) | Implementation kickoff |
| [13 GitHub/Wiki](docs/13-GitHub-Wiki.md) | Replacement and publication |
| [14 Security/assets](docs/14-Security-Assets.md) | Local resources and trust |
| [15 Tracking](docs/15-Tracking-Rules.md) | Progress and decisions |
| [16 Research](docs/16-Research.md) | Sources and Horizon inspiration |
| [17 Release](docs/17-Release.md) | Shipping gates |
| [Backlog](docs/tracking/BACKLOG.md) | Ordered executable tasks M0–M3 |
| [Decisions](docs/decisions/INDEX.md) | ADRs |
| [Wiki](wiki/Home.md) | Publishable documentation starter |

All cargo xtask commands are proposed interfaces to implement. This kit does not supply a functioning test runner or measured benchmark results. Missing implementations are NOT IMPLEMENTED, never PASS.
