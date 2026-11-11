# Sunset agent instructions

Read README.md, docs/tracking/PROGRESS.md, your task in docs/tracking/BACKLOG.md and the relevant domain document before implementation. The user's instructions and nearest applicable repository rules take precedence. Correctness, safety and maintainability outrank speed.

Use safe Rust and original WGSL. No adopted full renderer, first-party C/C++, unsafe production code, proprietary assets or secrets. Preserve target-version behavior. No simulation simplification for FPS.

Performance is a MUST: 3,000 FPS on BENCH-01 (docs/01-PRD.md section 4). Frame-loop code follows PERF-A01–A15: no per-frame allocation, shared lock, GPU resource creation, readback or log formatting. Changes to render, simulation, assets or threading run the bench quick gate; a regression is fixed or reverted before other work.

Keep edits narrow; inspect consumers before interface changes. Verify installed dependency APIs. Follow established patterns; no speculative frameworks or empty placeholders. Bound untrusted inputs, work queues, retries and memory. Preserve unrelated user changes.

First-party production Rust/WGSL contains no comments or commented-out code; explain design in Markdown. Preserve mandatory license notices. Tests may use explanatory comments where needed. Use Rust token-aware and WGSL-aware checks rather than regex that mistakes strings for comments. Generated shader assets must omit comments. Do not strip third-party notices. Treat dead-code warnings seriously; no blanket allowances to hide incomplete work.

Build foundation milestones before content expansion. Every behavior change normally has focused tests. Use exact parity cases and meaningful fault injection; never widen tolerances or refresh goldens just to pass. Missing private prerequisites are BLOCKED.

Read docs/15-Tracking-Rules.md for progress/decisions. Update tracking and handoff after each task with actual commands/exits/evidence. No fictional performance or test claims. Review final diff for unrelated changes/secrets/dead code.

Do not discard changes, rewrite history or force-push. Commits/pushes follow the owner's explicit authorized workflow. Maintain canonical Wiki sources with contract changes; label planned APIs.
