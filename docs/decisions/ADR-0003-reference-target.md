# ADR-0003 Target Java Edition 26.3

Status: PROPOSED · Origin: the old Sunset README (S01)

Context: the target must be pinned by artifact hashes before parity work. 26.3 is taken from the old README and is not yet verified against a local installation.

Decision (proposed): pin Java Edition 26.3 through T-005. If the version is not available locally or differs, T-005 returns BLOCKED and the owner decides; no automatic fallback to another version.

Acceptance: `target-lock.json` recorded with hashes; the owner marks this ADR ACCEPTED.
