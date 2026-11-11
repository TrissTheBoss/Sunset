# First Cline task

Use this task after adopting the handbook as the active repository specification ([13](13-GitHub-Wiki.md)). Afterwards every task comes from [BACKLOG](tracking/BACKLOG.md); this page is only the kickoff prompt.

> Execute backlog tasks T-001 through T-004 in order, one at a time, and stop after each task with a report. Before the first edit, read AGENTS.md, docs/01-PRD.md, docs/02-Parity.md, docs/10-Roadmap.md, docs/tracking/BACKLOG.md and docs/tracking/PROGRESS.md. Inspect the checkout and Git status; preserve user changes.
>
> Confirm the approved fresh-tree replacement state; do not reuse the old engine. Pin a real, installed and tested Rust toolchain and verify every dependency API against the installed crate version. Create only the crates the tasks name. Every first-party crate starts with `#![forbid(unsafe_code)]` and contains no comments.
>
> If the legal local game installation or the owner's hardware details are missing, implement the detection, return BLOCKED (exit 2) with an actionable message, and continue with the parts that do not depend on it.
>
> Do not implement content, LOD, mod host or renderer features yet. Do not claim any FPS number; measurement starts in M1.
>
> For each task: run the acceptance commands from the backlog, review the diff, and update PROGRESS.md, ENVIRONMENT.md and HANDOFF.md with exact commands and exit codes. Return the changed files, the evidence and the next READY task.

M0 acceptance: a reproducible reference manifest or explicit blocker, a real environment report, a clean original project skeleton, a functioning non-interactive doctor, the source lint, honest statuses and a recorded owner baseline (or BLOCKED with the owner action needed). A Markdown plan alone does not finish M0.
