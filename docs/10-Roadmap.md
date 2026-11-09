# Foundation-first roadmap

No invented delivery dates. Each milestone exits on evidence; dependent work stays blocked. The first content catalog begins only after M3. Small synthetic blocks/models/test entities are instrumentation, not a premature partial game.

| Milestone | Work | Exit evidence |
| --- | --- | --- |
| M0 Reference and reset | Recoverable repository replacement; target/hardware/toolchain lock; legal local provisioning; Cline tool smoke test | Verified manifest and provenance; clean project tree; first doctor result; unresolved choices recorded |
| M1 Measurement before speed | xtask runner, local oracle scaffolding, frame/tick metrics, synthetic fixtures, visual captures, schema/reporting | Injected faults detected; missing prerequisites nonzero; reproducible raw baseline |
| M2 Renderer and feasibility | Custom Rust/wgpu/WGSL, reference/greedy meshes, all drawable visibility classes, materials, jobs/cache, LOD/upscale prototypes, settings; shader-pack and mod-host spikes | Correctness toggles; real GPU timings; explicit pack/mod feasibility ADRs |
| M3 Foundation integration | Ownership/concurrency, registries, minimal protocol/session harness, storage transactions, module surfaces, UI/accessibility framework | Representative stress passes; safe reload/recovery; performance budgets; no fake gates |
| M4 World/logic parity | Exact generation, light, collision, ticks, player rules, integrated/dedicated server | Tick/worldgen/save differential cases including boundary seeds |
| M5 Content and advanced rules | Full inventory, entities/AI/combat/redstone/inventories/recipes/loot/dimensions | Expanded registry-level parity; command/admin coverage |
| M6 Interoperability and ecosystem | Real authentication, vanilla servers, client prediction, full packs, module SDK/examples | Authorized compatibility matrix; pack capability matrix; private test lane |
| M7 Complete parity closure | All remaining UI/audio/accessibility/save upgrade/operator/edge cases | No uncovered mandatory items, unresolved exceptions prevent full-parity claim |
| M8 Release hardening | Platform certification, performance tuning, packaging/security/wiki | Release checklist, raw benchmarks, reproducible artifacts and rollback |

Early protocol, pack and mod spikes in M2/M3 prevent architecture dead ends. They are narrow probes, not broad content implementation. Optimization continues after M3 because real content can expose bottlenecks. “All performance applications first” is interpreted as a complete measurable performance framework and representative proofs, not speculative optimization without evidence.

## Foundation gate

Before M4, demonstrate opaque/cutout/translucent/arbitrary geometry, animated entities/chests/particles/fonts, worker cancellation, stale-job rejection, reload rollback, GPU failure handling, block/entity visibility, greedy equivalence, LOD seams and upscale quality. Show frame and tick timing separately. A single empty cube scene is insufficient.

If a gate fails, minimize the failing workload and fix the boundary. Do not fill the content catalog to hide foundation shortcomings. No deadline pressure permits calling blocked tests passed.
