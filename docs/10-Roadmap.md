# Foundation-first roadmap

No invented delivery dates. Each milestone exits on evidence; dependent work stays blocked. The executable task list for M0–M3 is [BACKLOG](tracking/BACKLOG.md). Tasks for M4 onward are written into the backlog at the M3 exit, based on measured results.

| Milestone | Work | Exit evidence | Performance gate |
| --- | --- | --- | --- |
| M0 Reference and reset | Recoverable repository replacement; target/hardware/toolchain lock; legal local provisioning; data-generator inventory; BENCH-01 fixture; owner baseline; Cline smoke test | Verified `target-lock.json` or explicit BLOCKED; ENVIRONMENT complete; inventory counts in PARITY; `xtask doctor` result | Owner baseline client measured on BENCH-01 |
| M1 Measurement before speed | xtask runner, result schema, metrics crate, counting allocator, bench harness, visual capture scaffolding | All injected faults detected; missing prerequisites nonzero; reproducible raw baseline | BENCH-00 harness; noise calibrated |
| M2 Renderer core | wgpu/winit device and frame loop, synthetic terrain, reference and greedy meshers, geometry arenas, indirect draws, culling, passes, jobs, HUD batch; LOD/upscale prototypes only if the gate is green | Mesher equivalence; zero false culling; toggle equivalence; ADR-0005 capability tiers | **BENCH-01S ≥ 4,000 FPS**, PERF-A01–A15 checks green |
| M3 Foundation integration = VS-1 | Anvil reader, asset resolver, model baker, Fidelity lighting/fog/sky, tick thread + snapshot handoff, settings framework, minimal protocol/session harness, storage transaction design | VS-1 per PRD section 5; foundation gate below; VIS-BENCH-01 | **BENCH-01 ≥ 3,000 FPS** (PERF-T1, PERF-T2) |
| M4 World/logic parity | Exact generation, light engine, collision, ticks, player rules, saves, integrated/dedicated server | Tick/worldgen/save differential cases including boundary seeds | BENCH-01Q per merge; BENCH-05 PERF-T6 |
| M5 Content and advanced rules | Full inventory, entities/AI/combat/redstone/inventories/recipes/loot/dimensions, commands | Registry-level parity; command/admin coverage | BENCH-01Q per merge; BENCH-03 budgets |
| M6 Interoperability and ecosystem | Real authentication, vanilla servers, client prediction, full packs, module SDK/examples, Wasm runtime ADR, native WGSL packs | Authorized compatibility matrix; pack capability matrix; private test lane | BENCH-09; zero-listener ≤ 1% |
| M7 Complete parity closure | Remaining UI/audio/accessibility/save upgrade/operator/edge cases | No uncovered mandatory items; unresolved exceptions prevent a full-parity claim | BENCH-01 full |
| M8 Release hardening | Platform certification, LOD/upscaling finalization, PGO, packaging/security/Wiki | Release checklist, raw benchmarks, reproducible artifacts and rollback | All PERF-T gates on the release commit |

Early protocol and mod-host probes in M3 prevent architecture dead ends. They are narrow probes, not broad content implementation. Shader-pack (Iris/OptiFine) work does not start before M6 ([ADR-0004](decisions/ADR-0004-shader-pack-scope.md)). Optimization continues after M3, because real content exposes new bottlenecks; from M4 on, every merge holds the 3,000 FPS gate. "Performance first" means a complete measurable performance framework plus the P0 architecture of [05](05-Optimizations.md), not speculative optimization without evidence.

## Foundation gate (M3 exit)

Before M4, demonstrate opaque/cutout/translucent/arbitrary geometry, fonts and HUD, worker cancellation, stale-job rejection, reload rollback, GPU device-loss handling, resize/minimize, block visibility, greedy equivalence, and the tick/render handoff without shared locks. Animated entities, chests and particles are demonstrated with synthetic test models in M3; vanilla-exact entity rendering belongs to M5. Show frame and tick timing separately. A single empty-cube scene is insufficient.

If a gate fails, minimize the failing workload and fix the boundary. Do not fill the content catalog to hide foundation shortcomings. No deadline pressure permits calling blocked tests passed. If the M2 gate cannot be reached after profiling, open an ADR with the measured bottleneck before continuing; never lower the target silently.
