# DeepSeek V4 Pro in VS Code + Cline

## Verified versus assumed

Official DeepSeek documentation lists deepseek-v4-pro and tool-enabled thinking support as of research date. Official Cline documentation describes DeepSeek provider configuration and workspace Markdown rules. This does not prove a particular installed extension/provider version interoperates correctly. No credible evidence establishes a universal list of V4 Pro's personal weaknesses on this specific project. Avoid invented benchmark scores or claims of visual/compiler perfection.

Exploit tool-based coding, explicit contracts and focused test feedback. Guard against general model failure modes: plausible but nonexistent APIs, context drift, broad rewrites, incomplete verification, numeric-semantic mistakes and screenshot overconfidence. These are workflow risks, not asserted measured model deficits.

## Setup

Open the repository root in VS Code. Install/verify Cline and Rust/WGSL language tooling. Use the DeepSeek provider and select the actual deepseek-v4-pro model; store the key in the extension's secure settings, never Git. If native provider support is absent, verify a documented compatible provider path; never silently select a different model.

Record VS Code, Cline, provider route and model ID in the local environment report. Confirm [.clinerules/01-sunset.md](../.clinerules/01-sunset.md) is enabled; it is included in this repository. Persistent rules point to detailed docs rather than loading every file on every task. Do not assume a huge context window replaces durable records.

Use thinking/high effort for architecture, numeric semantics and concurrency when the actual integration supports it. DeepSeek's current pages contain differing effort-field details; validate effective provider behavior rather than copying untested API JSON into Cline. Temperature/top_p may be ignored in thinking/tool modes. No custom middleware is required merely to alter sampling.

Smoke test: read a harmless file; apply a small reversible fixture change; run a test; inspect failure; repair; inspect diff. Verify tool-call continuation, cancellation and provider error behavior. Record observations. Do not log secrets or demand hidden reasoning.

## One task loop

1. Read active progress, relevant rules/ADR and only nearby code/tests/config.
2. State desired behavior, invariants, acceptance command, affected files and unknowns.
3. Inspect public consumers before changing interfaces; verify installed APIs.
4. Write/reproduce one meaningful failing behavior case where practical.
5. Make the smallest coherent change; no unrelated formatting/refactors.
6. Run focused checks, then `cargo xtask check`. When the change touches render, simulation, assets or threading, also run `cargo xtask bench --suite bench-01q --gate` (BENCH-01S before M3) and record the numbers. Capture exit codes/artifacts.
7. Inspect diff, update progress/parity/decisions, report limitations.
8. Commit/push only under the project's explicit Git workflow authorization.

Keep tasks bounded: one backlog task, one behavior or one interface contract. Take the first READY task in [BACKLOG](tracking/BACKLOG.md) whose dependencies are VERIFIED. Treat 3–5 files as a planning heuristic, not an excuse to split an atomic change incorrectly. After two failed patch attempts revisit assumptions and diagnostics. Never disable checks, weaken types or widen tolerances to claim success.

## Extra care for difficult work

Numeric parity: show reference vectors for negative coordinates, extreme integers, casts, NaNs, random state and ordering; compare outputs, not explanations.

GPU work: verify layouts/alignment/binding limits/depth convention; validate shaders and use debug buffers. A plausible screenshot is insufficient.

Concurrency: write ownership and cancellation/commit order explicitly in docs; reproduce races with controlled scheduling and revisions, not sleeps.

Performance: obtain a raw baseline before changing; never infer FPS from language choice or a successful build. Check every hot-path change against PERF-A01–A15: no allocation, lock, resource creation, readback or log formatting inside the frame. When a quick gate regresses, revert or fix before anything else.

Before context compaction/end of task update HANDOFF.md with goal/constraints/files/decisions/checks/failures/next action. Keep private paths/tokens out. Resume from evidence, not memory. No automatic remote deletion, force push, baseline approval or credential handling.
