# Automated visual validation

Validate assets structurally first: references resolve, dimensions/animation indices/durations match, UV rotations/model inheritance/font glyphs/pack precedence are correct. A valid PNG does not prove it is bound to the correct face.

Render a controlled gallery in local vanilla and Sunset. Freeze camera matrices/FOV/output resolution/GUI scale/time/weather/biome tint/light/pose/animation/particle seeds and pack hash. Capture real presentation as well as headless targets. Record OS scale/color management and use a known sRGB path.

Inventory every block/state/model binding. Cases include six faces, slabs/stairs/fences/doors/multipart models, water/lava/glass/foliage, tinted faces, chests, entity poses/skins, particles, held/dropped items, breaking overlays, hotbar/UI, Unicode fonts, sky/fog/weather. Add all target-specific features. Lighting covers sun/dark/AO corners/light boundaries/chunk seams. Packs cover non-square/high-resolution/animated/missing assets and reloads. Proprietary captures stay private.

## Comparators

1. Reject metadata/dimension/camera mismatches before image metrics.
2. Compare Sunset identity/depth/UV/normal/light debug buffers to analytic synthetic expectations.
3. Require exact golden pixels for deterministic same-backend fixtures proven stable.
4. Across reference/backends compute regional absolute error, SSIM, edge displacement and mismatch area in specified color spaces.
5. Segment faces/text/HUD/silhouettes/background; report worst regions rather than only whole-frame average. Missing expected geometry/texture pixels are hard failures.
6. Compare sequences for timing/flicker/ghosting/LOD transitions.

Initial synthetic calibration proposals: SSIM >=0.995, at most 0.1% opaque-region pixels above 3/255 absolute channel error, silhouette displacement <=1 pixel. These are not established parity tolerances. M1 measures repeated vanilla noise per backend and freezes per-case limits before acceptance. Structural binding checks remain mandatory. Masks require proven nondeterminism and bounded documented scope; never hide a bug with a broad mask.

Enhanced LOD paths cross boundaries near caves/towers/overhangs/forests/shores/recent edits. Require no cracks, unknown-terrain reveal or stale geometry beyond the declared deadline; freeze per-band popping/edge budgets. Upscaling compares native Sunset with moving foliage/water/thin geometry/particles/transparency/disocclusions/cuts. HUD remains native.

Failures save reference/actual/heatmap/region overlay/contact sheet and metadata with first mismatch. Deterministic metrics decide PASS; a vision model may help triage but is never the sole judge.

Goldens are keyed by reference/pack/backend. Runner never auto-approves replacements. Baseline changes require old/new images, reason and a separate reviewed commit. Routine testing is unattended; accepting a new baseline is a deliberate decision. Without an oracle, structural tests cannot certify vanilla appearance.
