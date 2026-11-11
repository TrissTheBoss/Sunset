# Unattended verification

## Boundaries and commands

After one-time provisioning of legal local files, Java, GPU runner and needed account authorization, routine tests run without input. Purchase/install, terms acceptance, MFA, expired credentials and baseline approval cannot honestly be guaranteed unattended. Missing prerequisites produce BLOCKED, not a prompt loop or false pass.

These are interfaces to implement, not included executable tools:

| Command | Contract |
| --- | --- |
| cargo xtask doctor --non-interactive | Validate toolchain/reference/assets/GPU/private paths |
| cargo xtask test --suite unit --non-interactive | Deterministic unit/property tests |
| cargo xtask parity --suite all --non-interactive | Vanilla/Sunset differential cases |
| cargo xtask visual --profile fidelity --non-interactive | Images/metrics/diffs |
| cargo xtask bench --suite standard --non-interactive | Benchmark protocol |
| cargo xtask bench --suite bench-01 --gate --non-interactive | REQ-03 gate: exit 1 when PERF-T1/PERF-T2 fail |
| cargo xtask bench --suite bench-01q --gate --non-interactive | Per-merge quick gate (BENCH-01S before M3) |
| cargo xtask inventory --non-interactive | Data-generator inventory and PARITY denominators |
| cargo xtask lint --non-interactive | fmt, clippy, forbid-unsafe, token-aware comment check for Rust and WGSL, release-profile check |
| cargo xtask check --non-interactive | Fast local loop: lint + unit + synthetic fixtures; what an agent runs before every handoff |
| cargo xtask verify --tier public --non-interactive | Distributable-fixture checks |
| cargo xtask verify --tier private --non-interactive | Oracle/GPU/account-dependent checks |
| cargo xtask report --run RUN_ID | Immutable evidence summary |

Exit 0 = all required checks pass; 1 = assertion/regression; 2 = blocked/configuration; 3 = runner crash; 4 = timeout/cancellation. Required skips are nonzero. Optional unsupported features are NOT APPLICABLE with declared scope.

## Runner

A Rust runner creates isolated profiles, allocates loopback ports, launches children with argument arrays, waits for authenticated readiness, injects traces and captures outputs. Control uses named pipe or loopback with per-run random credential, versioned bounded messages. No exposed unauthenticated control interface.

Default case timeout 120 seconds; long cases declare exceptions and suite caps. At most two infrastructure retries with backoff; never retry an assertion into a pass. Retain first-failure artifacts. Cleanup process groups/temp profiles on success/failure/cancellation. Orphan processes and resource leaks fail.

Use original private Java adapters only where necessary to drive/capture local vanilla. Independently check adapters against black-box behavior. Unreliable instrumentation is BLOCKED, not parity. Sunset is not its own behavioral oracle.

## Suites

0. Performance: allocation-free frame loop, creation counters, draw/pass/submit counts, timing-ring correctness, bench gate exit codes.
1. Unit/property: arithmetic, RNG, IDs/palettes, codecs, collision, meshing and canonical serialization.
2. Differential: ticks, worldgen, inventory, commands, AI, fluids, redstone, saves and client prediction.
3. Metamorphic: save/load identity, thread-count invariance, optimization-toggle equivalence and atomic reload.
4. Network: authentication, compression/encryption, version mismatch, malformed input, chunk updates, teleports/respawn, signing, reconnect and network disturbance.
5. Fuzz: protocol, NBT/region, JSON/zip/fonts/images/module manifests with input/decompression caps.
6. Persistence: disk full, interrupted writes/migration, backup recovery, unknown records and downgrade refusal.
7. Lifecycle: 100 load/unload cycles, repeated reloads, device recreation, long soak and shutdown.
8. Mod host: zero/loaded hooks, bounded stress, timeout/capability denial and order.
9. UI/audio/accessibility: focus, settings, translations, subtitles, pitch/gain/position, narration and device changes.

## Case and result schema

Case fields: id, requirement_ids, parity_ids, target_lock_hash, seed, initial_state_hash, packs, profile, tick-indexed actions, observation phases, comparator/tolerances, capabilities, timeout_seconds, privacy_class and expected_outcome. Property failures retain seeds/shrunk input.

Result fields: schema version, case ID, commit/dirty flag, executable/reference/fixture hashes, timestamp, OS/toolchain/driver/backend, status/time, expected/actual, artifact paths/hashes. Redact secrets at collection. Emit JSON/JUnit/Markdown. Link evidence to immutable tested commits.

Public PR CI: `cargo xtask check`. Trusted GPU lane: `bench-01q --gate` for every change touching render, simulation, assets or threading (PERF-A15). Untrusted PR code must not run on privileged GPU/account runners. Trusted revisions use isolated private oracle and controlled GPU lanes. Software adapters validate API behavior, not hardware FPS. Nightlies expand seeds/fuzz/soaks; releases require all mandatory lanes/platform tiers.

Harness acceptance requires injected allocation in the frame loop, injected 2 ms stall on the render thread, injected wrong texture, dropped tick, malformed packet, stale mesh, changed oracle hash, memory-limit breach and timed-out child. Every fault must fail for the intended reason with useful evidence. A harness that only passes has not proved detection ability.
