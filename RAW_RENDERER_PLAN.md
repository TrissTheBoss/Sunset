# Raw wgpu Renderer — Plan

**Status:** proposal (not yet implemented)
**Date:** 2026-11-08

This document plans a minimal, custom raw renderer to escape the fixed per-frame
CPU overhead of Bevy's ECS + render graph, which currently caps the game at
~460 FPS regardless of scene complexity. It reuses `sunset-core` (the
authoritative world/simulation) so it is a rendering-adapter swap, not "mock
gameplay" (PRD 11.1).

---

## 1. Objective

Reach a **stable 3000+ FPS** on the `building-yard` fixture (AMD Radeon RX 6800 XT,
Vulkan), while keeping the same world, simulation, player controller, and target
textures. The Bevy adapter remains the reference implementation for correctness;
the raw renderer is a performance experiment.

## 2. Context and the measured bottleneck

Measured (Windows 11, Ryzen 5 5600X, RX 6800 XT, Vulkan):

| State | FPS | Notes |
| --- | --- | --- |
| Original dev build (`opt-level=0` everywhere) | ~90 | CPU-bound, unoptimized Bevy pipeline |
| Optimized deps (`opt-level=3` for Bevy/wgpu) | ~350 | |
| + greedy meshing + MSAA off + combined mesh | ~460 | mesh dropped 40k → **448 triangles** |

The mesh is now 448 triangles and the GPU is essentially idle, yet FPS is stuck
at ~460 (~2.2 ms/frame). The remaining time is **Bevy's fixed per-frame CPU cost**:
ECS scheduling, render-graph traversal, entity extraction, and command encoding.
These run every frame regardless of how trivial the scene is. The geometry was
never the bottleneck.

**Conclusion:** 3000 FPS (0.33 ms/frame) requires removing that fixed cost, which
means a raw render loop that does the minimum work per frame.

## 3. Scope and non-goals

**In scope**

- A raw `winit` + `wgpu` render loop (no Bevy ECS, no render graph, no materials).
- One opaque pass + one transparent (glass) pass, one draw call each.
- Reuse `sunset-core` world/simulation/player/physics unchanged.
- Reuse the greedy meshing algorithm and the texture-atlas loader.
- First-person camera, mouse look, WASD + jump, block targeting and break/place.
- A minimal FPS readout (console or a tiny overlay).

**Out of scope (deferred)**

- Text rendering / full HUD parity (hotbar, etc.) — needs a glyph atlas or `egui`.
- Networking, worldgen, native save-data probes.
- Making the raw renderer the production path — it is an experiment until proven.
- Any claim of vanilla visual parity beyond "same mesh, same textures".

## 4. Architecture

The raw renderer is a new binary crate that depends on `sunset-core` and does its
own windowing, GPU setup, and rendering. It keeps the existing `sunset` Bevy
crate untouched so the two can be A/B compared.

```
sunset-core (world/simulation/physics)  <-- reused unchanged by both
      ^                    ^
      |                    |
sunset (Bevy)        sunset-raw (new)
reference,           winit loop + wgpu
unchanged            pipeline + camera
```

The raw renderer's per-frame work is:

1. Poll `winit` events (keyboard/mouse).
2. Step the simulation if a fixed tick elapsed (20 Hz, as today).
3. Compute the view-projection matrix from the player camera.
4. Write one uniform, encode one render pass, submit, present.

That is the entire frame. No scheduling, no extraction, no graph.

## 5. Dependencies

Pin the same versions Bevy 0.20 uses so behavior (and any future asset/format
work) matches:

- `wgpu = "30.0.1"` (default features: dx12 + vulkan + metal)
- `winit = "0.30.13"`
- `image` (already used by the atlas loader)
- `glam` (already used by `sunset-core`)
- `sunset-core` (path dependency)
- `bytemuck` (for the uniform/vertex `Pod` types)

## 6. GPU pipeline (detail)

### 6.1 Vertex format

Unlit, textured, per-face shaded. Drop the normal (not needed without lighting).

| Attribute | Type | Bytes |
| --- | --- | --- |
| position | float32x3 | 12 |
| uv | float32x2 | 8 |
| color | float32x4 (face shade) | 16 |

36 bytes/vertex, a single interleaved buffer.

### 6.2 Uniform

A single 4x4 view-projection matrix (64 bytes), updated once per frame.

### 6.3 Textures

The packed block atlas as a `Rgba8UnormSrgb` texture, sampled with **nearest**
filtering (established earlier to keep 16×16 textures crisp). Same atlas image
the Bevy renderer uses.

### 6.4 Shader (WGSL)

```rust
@group(0) @binding(0) var<uniform> vp: mat4x4<f32>;
struct VsOut { @builtin(position) clip: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) col: vec4<f32> };
@vertex fn vs_main(@location(0) pos: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) col: vec4<f32>) -> VsOut {
    var o: VsOut;
    o.clip = vp * vec4<f32>(pos, 1.0);
    o.uv = uv;
    o.col = col;
    return o;
}
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.uv) * in.col;
}
```

### 6.5 Pipeline state

- Primitive: triangle list, cull back.
- Depth: opaque pass writes + tests; transparent pass tests only (no write).
- Blend: transparent pass `ONE, ONE_MINUS_SRC_ALPHA`.
- Multisample: **off** (`sample_count = 1`) — no MSAA texture.

## 7. Render passes

1. **Opaque pass** — clear color, write depth, draw the opaque world mesh.
2. **Transparent pass** — same depth attachment (test, no write), alpha blend,
   draw the glass mesh.

Two draw calls total.

## 8. What is reused vs. rewritten

### Reused unchanged (`sunset-core`)

- `World`, `Chunk`, `BlockState`, `block_def`, `occludes`, `is_transparent`.
- `physics::{raycast, move_entity, Aabb}`.
- `simulation::Simulation`, `player::Player`, `intent::PlayerIntent`.
- `fixture::building_yard`, `save::{from_json,to_json,write_atomic}`.

### Adapted (copied, then decoupled from Bevy)

- **Greedy mesher** (`sunset::renderer::greedy_face` + `FACE_*` tables) — the
  sweep/merge logic is already renderer-agnostic (it uses `glam::Vec3` and
  `BlockState`). Re-target it to emit `Vec<Vertex>` + `Vec<u32>` instead of a
  Bevy `Mesh`.
- **Atlas loader** (`sunset::mcassets::load_assets`) — already Bevy-independent;
  it returns `image::RgbaImage` + a UV map. The raw renderer uploads that
  `RgbaImage` straight to a `wgpu` texture.

> Consolidation: if the raw renderer wins, promote the mesher and atlas loader
> into a shared `sunset-render-common` crate so `sunset` and `sunset-raw` share
> one implementation. Do this *after* proving the approach, not before (PRD
> warns against premature abstraction).

### New

- Window + event loop, wgpu device/surface/swapchain, pipeline, buffers.
- Camera math (view-projection from yaw/pitch/position).
- Input capture (mouse grab, key state).

## 9. Input, camera, targeting, HUD

- **Input** — `winit` `WindowEvent::KeyboardInput` / `CursorMoved`, plus
  `set_cursor_grab`. Build the same `PlayerIntent` the Bevy adapter builds.
- **Camera** — eye = `player.pos + (0, eye_height, 0)`; view from yaw/pitch;
  projection is a standard perspective (same FOV as the Bevy camera).
- **Targeting** — `physics::raycast` from the eye along the look direction.
  Highlight is a 12-segment wireframe box (a tiny static line vertex buffer),
  or deferred.
- **HUD** — print FPS to the console each second for the benchmark; a full text
  overlay is deferred (glyph atlas is a separate chunk of work).

## 10. Milestones

Each milestone ends with a buildable, measurable result.

- **M0 — Window + clear color.** `winit` loop + `wgpu` surface; render a solid
  color with `PresentMode::Immediate`. Validate the empty loop reports thousands
  of FPS (this alone confirms the Bevy overhead is gone).
- **M1 — Render the world.** Port the greedy mesher + atlas loader, upload mesh
  and texture, render the opaque pass with depth. Verify visual parity with the
  Bevy renderer (same 448-triangle scene).
- **M2 — Camera + input.** View-projection from player state; reuse simulation +
  physics for WASD/mouse-look/jump. Verify you can walk the fixture.
- **M3 — Transparent pass.** Glass (depth-test no-write + alpha blend).
- **M4 — Target + edit.** Raycast highlight + break/place via the simulation.
- **M5 — Benchmark + compare.** Run the fixture, record FPS (avg + 0.1% low),
  compare against the Bevy adapter on the same machine.
- **M6 — (optional) HUD.** Minimal FPS overlay or `egui` integration.

## 11. Success criteria

- **3000+ FPS** (avg) on the `building-yard` fixture at 1080p, RX 6800 XT,
  `PresentMode::Immediate`, no MSAA.
- Rendering visually matches the Bevy adapter for the same world + textures.
- Movement, targeting, and block edit behave identically (same `sunset-core`
  path).
- A written benchmark record (revision, fixture, config, FPS) per PRD 7.3.

## 12. Risks and unknowns

1. **wgpu 30 API specifics** — render pass/surface/pipeline details differ from
   the older tutorials; expect to read the `wgpu` docs while implementing.
2. **winit 0.30 `ApplicationHandler`** — event-loop API (no `run(move |..|)`).
3. **Texture format** — the atlas is `Rgba8UnormSrgb`; the sampler + shader must
   match or colors shift.
4. **Present overhead** — even `Immediate` has some driver/swapchain cost; the
   true ceiling may be ~2000–4000 FPS rather than "unbounded".
5. **Depth correctness** — the opaque pass needs depth to avoid sorting
   artifacts; transparent (glass) needs correct depth handling.
6. **HUD text** — non-trivial if a full HUD is wanted (glyph atlas/egui).
7. **Duplication** — the mesher/atlas are temporarily duplicated until the
   shared-crate consolidation.

## 13. Decision log

- **New crate `sunset-raw` rather than modifying `sunset`** — keeps the working
  Bevy app for A/B comparison and avoids regressing gameplay.
- **Reuse `sunset-core` unchanged** — the world/simulation is renderer-agnostic;
  swapping only the presentation layer is the smallest correct change.
- **Copy the mesher/atlas now, consolidate later** — avoids a premature
  cross-crate refactor; extract to `sunset-render-common` only if the raw path
  is kept.


