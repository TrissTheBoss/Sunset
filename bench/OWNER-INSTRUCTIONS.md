# bench-01 owner instructions

Two owner actions unblock the M0 baseline (PERF-T3). Run them in a vanilla Java Edition 26.3 install.

## 1. Create the bench-01 save

1. Create a new singleplayer world named `bench-01`:
   - Seed: `sunset`
   - World type: default
2. Pre-generate a radius of 16 chunks around spawn (any pre-generation method; ensure spawn-area chunks are generated).
3. Apply game rules:
   - `/gamerule doDaylightCycle false`
   - `/time set 6000`
   - `/gamerule doWeatherCycle false`
   - `/weather clear`
   - `/gamerule doMobSpawning false`
4. Remove all entities and block entities in view (terrain, foliage and water only).
5. Locate the save and copy it into `../sunset-private/bench-01/` (outside the repository).

## 2. Record the baseline client measurement

1. Load the `bench-01` save.
2. Set: render distance 8, simulation distance 8, FOV 70, VSync off, frame cap off, 1920×1080 borderless, GUI scale 2.
3. Fly as spectator at spawn, eye 1.62 blocks above the surface.
4. Measure median FPS over 120 s (60 s warm-up), 5 runs, with the original settings that produced the ~3,500 FPS report.

Record in `docs/tracking/ENVIRONMENT.md` → "Owner baseline": client version, loader/mods/configs, JVM flags, scene/seed/camera, render/simulation distance + FOV, resolution/window mode/VSync/cap, and measured median FPS.

If the settings differ from PRD section 4.2, amend ADR-0002 before the M2 gate.
