# ADR-0006 Thread model and lock-free snapshot handoff

Status: PROPOSED · Origin: handbook revision 2; accept with T-305

Decision (proposed): threads per [03](../03-Architecture.md): render/main, tick, workers (cores − 2), IO, later network. Tick → render uses a lock-free triple buffer or an atomic swap of immutable snapshots; the render thread interpolates by the partial tick. Workers → render uses bounded lock-free queues. The render thread never takes a lock shared with another thread.

Why: any contention on the render thread is visible at 333 µs per frame; tick spikes (redstone, entities) must not reach frame times.

Evidence required: contention counters at zero on BENCH-01 and BENCH-05; interpolation correctness test.
