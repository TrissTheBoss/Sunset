# Testing and performance

The planned Rust xtask runner will perform preflight, unit/property, differential vanilla, visual, protocol, fuzz, persistence and benchmark suites. Its commands must first be implemented. Missing legal assets/GPU/authentication block relevant checks.

Visual validation combines structural asset checks, controlled reference captures, regional pixel/edge metrics and temporal tests. Baselines cannot approve themselves.

Performance is a MUST: at least 3,000 FPS on the named benchmark BENCH-01 in Vanilla Fidelity at native resolution on the owner's machine, gated on every merge once the renderer exists. 4,000 FPS is a stretch goal. The owner's hardware is not yet verified. Reports must disclose native/enhanced settings, actual frame times, tail latency, simulation, memory, warmup and repeated runs. Empty scenes and frame generation do not prove gameplay throughput.

Reports link immutable code/reference/fixture hashes to raw artifacts. Proprietary captures and tokens remain private.
