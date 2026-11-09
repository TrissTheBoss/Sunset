# Sunset persistent rules

Follow AGENTS.md. At task start read docs/tracking/PROGRESS.md and HANDOFF.md, then only the relevant PRD/parity/architecture/ADR sections.

Use the verified deepseek-v4-pro route; never silently substitute a model. Verify actual installed APIs and tool outputs. Work one bounded acceptance gate at a time. Inspect → reproduce/specify → implement → verify → inspect diff.

No target-version drift, FPS promises without raw measurements, shader-pack compatibility shortcuts, skipped tests marked passed, or bulk content before M3. Distinguish Fidelity from Enhanced and implementation from verification.

Keep first-party production Rust/WGSL comment-free and safe, with design details in Markdown. Preserve licenses. No success-returning stubs, unused abstractions, blanket lint suppression or secret/proprietary uploads.

Use docs/07-Automated-Testing.md and docs/09-Benchmarks.md. Proposed xtask commands do not exist until implemented. After two failed patch attempts revisit assumptions. Maintain durable task/evidence/decision state before stopping.
