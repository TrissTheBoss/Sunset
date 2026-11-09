# Risks and unresolved choices

| ID | Risk / uncertainty | Likelihood / impact | Mitigation / decision gate | Owner / status |
| --- | --- | --- | --- | --- |
| R-01 | 4,000 FPS unattainable in desired scene | Unknown / high | Capture hardware and BENCH-01 before promise; M1 | Unassigned / OPEN |
| R-02 | Target reference metadata incomplete | High / high | Verify exact 26.3 install/hashes; M0 | Unassigned / OPEN |
| R-03 | GLSL pack compatibility beyond chosen backend | High / high | Small licensed corpus and frontend spike; M2 | Unassigned / OPEN |
| R-04 | Mod sandbox overhead | Medium / high | Batched ABI and measured no-op/real modules; M2 | Unassigned / OPEN |
| R-05 | Rare parity/order bugs | High / high | Inventory, trace minimization, oracle fuzz; ongoing | Unassigned / OPEN |
| R-06 | Local import mistaken for legal clearance | Medium / high | Provenance/distribution review; pre-release | Unassigned / OPEN |
| R-07 | GPU/driver visual nondeterminism | Medium / medium | Per-backend calibrated fixtures; M1 | Unassigned / OPEN |
| R-08 | AI context drift/fictitious test results | Medium / high | Durable records and machine evidence; each task | Unassigned / OPEN |
| R-09 | Premature optimization delays usable game | High / high | Gate candidate techniques on measured benefit; M3 | Unassigned / OPEN |
| R-10 | Save loss/migration incompatibility | Medium / critical | Copies/backups/atomic writes/crash tests; M3 | Unassigned / OPEN |
| R-11 | Server anticheat policy differences | Unknown / high | Authorized configuration matrix; no universal guarantee; M6 | Unassigned / OPEN |
| R-12 | Private runner/account unavailable | Medium / high | Explicit BLOCKED, safe public lane; M1 | Unassigned / OPEN |

Do not convert unknown hardware, current pack support or model integration into invented facts. Add trigger, evidence and next decision date once a task owner is assigned.
