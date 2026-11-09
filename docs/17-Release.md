# Release checklist

A release is blocked by any unmet mandatory item. Preview labels may narrow scope honestly; they cannot claim full parity.

- Exact target reference/toolchain/dependency/build hashes recorded.
- Public and private suites pass on the actual release commit; no mandatory skip.
- Complete advertised inventory covered; known differences disclosed and critical ones resolved.
- Correctness passes with optimizations toggled; GPU/backend capability matrix certified.
- BENCH-01 evidence for any 4,000 FPS claim; native and enhanced results distinct.
- Frame/tick tails, memory, streaming, reload and soak budgets pass.
- Visual/asset/UV/model/font/animation checks and deliberate fault detection pass.
- Authentication, server permissions, protocol and authorized compatibility cases pass.
- Resource/data packs and shader compatibility explicitly scoped; unsupported features visible.
- Module APIs/examples/version policy tested; trusted extensions clearly separated.
- Save upgrades, backups, interrupted writes and rollback tested on copies.
- No first-party production comments/dead code/debug placeholders; required notices retained.
- No proprietary resources, private captures, tokens or personal data in tree/history/artifacts.
- Accessibility/localization/settings/recovery verified.
- Packages, checksums, SBOM, licenses, provenance and rollback instructions ready.
- GitHub tag/notes and version-matched Wiki published and link-checked.
- Evidence records actual commands/exits and immutable run/artifact identities.

Definition of done: requested behavior works in declared scope, user data preserved, checks passed or release blocked, evidence and docs current, no unrelated temporary work.
