# Assets, security and distribution

## Local resources

Discover supported launcher roots read-only or accept an explicit path. Validate the pinned client/version metadata and asset index/hash; resolve textures/models/sounds/fonts/languages from the user's installed legally obtained copy. A jar's presence is not proof of purchase. Legitimate account entitlement/session checks belong in the login flow where available; do not claim filesystem detection proves licensing.

Never bundle/upload proprietary jars, textures, sounds, fonts, extracted tables or private oracle screenshots. Do not modify the original install. Store derived local caches separately with source/version/pack hashes and atomic writes. All private material (target lock, inventory reports, fixture saves such as `bench-01`, vanilla captures, goldens) lives under the private root, default `../sunset-private/` outside the repository, overridable with `SUNSET_PRIVATE_ROOT`. `.gitignore` additionally excludes `private/`, `target/` and `runs/` as a safety net. Missing assets produce actionable errors; authored fallback materials are development-only and cannot pass Fidelity.

Reject zip traversal/absolute paths/symlink escapes, cap file counts/decompressed sizes/image dimensions and validate parsers. Cache content is untrusted too. Hashes detect mismatch, not copyright permission.

Minecraft's EULA/usage guidelines govern redistribution and branding; local extraction alone does not establish legal permission for every implementation technique. Assess applicable terms and clean-room/provenance practices before public distribution. Keep original code/tests; do not publish decompiled source. This handbook is not a legal clearance. Include an independent unofficial-product notice and required third-party licenses.

## Accounts/network

Use legitimate supported Microsoft/Minecraft authentication and established cryptography. Store tokens in OS credential storage, never source/logs/fixtures. No password collection in custom game UI if the supported flow uses browser/device authorization. Validate server identity/session behavior; respect signing/security requirements of the target. Token expiry is an explicit blocked/re-authentication state, not bypass.

Bound packet sizes, recursion, allocations, decompression, queues and timeouts. Handle invalid state transitions, backpressure, cancellation and reconnect. Fuzz codecs/storage/images/fonts. Authenticate local test control interfaces and bind loopback only.

Compatibility testing uses owned/authorized servers. Match movement, collision, packet cadence/order and prediction; investigate flags as parity bugs. No anti-detection fingerprint spoofing, account bypass or guarantee against every server policy. Test permitted anticheat versions/configurations against the genuine client and report differences honestly.

## Modules/builds/saves

Sandboxed modules get explicit capabilities and resource budgets. Trusted source extensions are clearly labeled. Never execute untrusted PRs on runners holding private assets/tokens. Dependency review includes unsafe/native boundaries, licenses, advisories and supply chain. Pin versions, provide SBOM/provenance/checksums and separate debug artifacts from releases.

Save to temporary sibling, flush as needed, atomically replace using platform-appropriate semantics and retain recoverable backup. Test disk full/crash during writes and migration. Do not overwrite the user's original worlds during testing; operate on copies. Refuse unsupported downgrades.

No telemetry by default. Reproduction bundles are opt-in, redact tokens/usernames/paths/server addresses as configured, and separate proprietary assets. Memory safety does not remove the need for limits, consent and recovery.
