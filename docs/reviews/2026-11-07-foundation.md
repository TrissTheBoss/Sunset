# Independent foundation review — 2026-11-07

**Reviewer:** `gpt-6.1-sol`, medium reasoning. **Disposition:** no unresolved blocker identified for the reviewed documentation and non-playable source scaffold. This is not gameplay, compatibility, security implementation, or release-publication approval. The orchestrator reviews this report and makes the publication decision.

## Scope and checks

Read `CORE_PHILOSOPHY.md`, `AGENTS.md`, PRD, parity matrix, architecture, release policy, source/decision/action records, role documentation, agent templates, Rust manifests/source/lockfile, license, README, changelog, CI and source-prerelease workflow.

- Independently passed `cargo +stable fmt --all -- --check`, `cargo +stable clippy --workspace --all-targets --locked -- -D warnings`, and `cargo +stable test --workspace --locked` on installed Rust 1.99.1. There are zero unit tests and zero doctests; these checks establish scaffold compilation and lint hygiene only.
- Counted 98 matrix acceptance rows with 98 unique IDs. All remain unverified. Exact artifact hashes, protocol identity, complete target inventory, supported hardware/platforms, and behavioral evidence remain future gates, not achieved results.
- Inspected four role templates: exactly three `gpt-6-luna` medium coders (`requirements`, `framework`, `governance`) and one `gpt-6.1-sol` medium reviewer. Global defaults use Luna/medium with concurrency four. Required standalone role fields are present and the simple TOML syntax was inspected. CLI parsing/loading/trust was not executed; Python/`py` was unavailable. Local `.codex` installation is blocked and explicitly disclosed; templates under `config/codex` are not active configuration by themselves.
- Reviewed source-prerelease workflow structurally. It validates input syntax, exact checkout SHA, main ancestry, existing-tag commit equality and crate/tag version agreement; runs checks; packages source; labels the release non-playable. The workflow was not dispatched during review. Remote publication and mapped `.codex` contents remain unverified.

## Findings and distinct follow-up

| Severity | Finding and impact | Author remedy and follow-up |
|---|---|---|
| High | `.github/workflows/foundation-prerelease.yml` initially allowed an existing release tag to point elsewhere than the SHA named in its source archive/body. | Author added peeled-tag commit equality and main ancestry checks. Re-read the final workflow and confirmed the safeguards. |
| Medium | PRD release gate initially checked P1 rows only and used severity language inconsistent with the matrix, permitting an incomplete full-parity claim. | Author now requires every applicable baseline priority, consistent S0–S3 mismatch rules and explicit treatment of unknown applicability. Re-read PRD sections 5/9 and matrix claim/release rules. |
| Medium | `Cargo.toml` initially advertised MIT OR Apache-2.0 while the license/decision selected MIT. | Author changed manifest metadata to MIT. Re-read manifest, license and D-006. |
| Medium | Initial persistent role drafts contained four coder roles rather than the requested three. | Author replaced them with requirements/framework/governance plus reviewer. Re-read templates and workflow role table. |
| Medium | Initial checks failed for missing `Cargo.lock` and formatting. | Author generated the lockfile and formatted. All three checks passed independently afterward. |
| Low | Intermediate matrix edits duplicated target IDs and mixed optional shader compatibility with baseline graphics coverage. | Author removed duplicate IDs, expanded concrete mechanics criteria, separated baseline `/posteffect` and vanilla graphics from optional loader support, and added required Sunset AA enhancement criteria with vanilla-preserving preset and safe fallback. Re-counted IDs and re-read FR-053a/b and EXT-AA-01/02. |

The reviewer did not edit author deliverables or implement any fixes.

## Philosophy audit

| Commitment | Design review result and evidence |
|---|---|
| Vanilla compatibility first; performance second | Aligned in PRD sections 3/5/7/9, matrix claim rules and PERF-02. No parity/performance result is claimed. |
| Ordinary authenticated unmodified servers | Aligned in PRD FR-010–015, NET-02/03/07 and AUTH-01. Authentication is unimplemented; service access and account eligibility remain external constraints. |
| Lifetime free functionality | Explicit in PRD section 1 and release policy. No payment/subscription implementation exists. MIT permits third-party commercial redistribution; the first-party project commitment is separately documented. |
| Local user asset import; no bundled copyrighted game assets | PRD FR-001/052, architecture asset boundary, PACK-02 and DIST-01 require local provenance and no redistribution. Source inventory contains only project scaffolding and no game assets. Import itself is unimplemented. |
| Honest evidence and limitations | README and source-prerelease text explicitly say non-playable; matrix entries remain unverified; exact target gaps are disclosed. The PRD does not claim a complete captured inventory and requires that inventory before implementation planning is considered ready. |
| Unofficial, independent status | Explicit in philosophy and AGENTS. No inspected material claims endorsement or publisher affiliation. |

## Remaining limits

Final follow-up confirmed: PRD section 5, the matrix claim rule and EXT-AA-01/02 now distinguish required product enhancements from vanilla baseline parity, retain their product-release requirement, and render their classification within the existing five-column table. No outstanding finding remains for publication of the reviewed non-playable scaffold.

This documentation establishes direction and acceptance gates. It does not demonstrate 100% target coverage, a playable client, functioning import/authentication/multiplayer, safe world persistence, or improved performance. The reference manifest, exhaustive oracle-derived feature catalog, authorized auth/import specification, threat model, support matrix and measured budgets must be completed before those claims. CI uses moving stable rather than a reproducibly installed exact compiler. No workflow run, remote source/release integrity check, or CLI role-loading test was performed by this reviewer.
