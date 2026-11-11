# Research and source ledger

Checked 9 November 2026. These are primary sources or the project's own statements. Design proposals in this handbook are not measured outcomes. Links may change; pin exact dependency/reference versions during implementation.

| ID | Source | Use and limit |
| --- | --- | --- |
| S01 | [Sunset README](https://github.com/TrissTheBoss/Sunset/blob/b969f6974e1b78490ae542ce55d8e07736eca8b8/README.md) | Existing target 26.3 and prototype context; not inherited evidence |
| S02 | [Horizon documentation](https://playhorizon.dev/docs) | Original Rust rewrite, local files and differential testing as project-reported approach |
| S03 | [0 FPS meshing](https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/) | Cached voxel meshes and face reduction; simplified model |
| S04 | [0 FPS meshing part 2](https://0fps.net/2012/07/07/meshing-minecraft-part-2/) | Greedy/monotone tradeoffs and material grouping; not a guarantee of speed |
| S05 | [NVIDIA geometry clipmaps](https://developer.nvidia.com/gpugems/gpugems2/part-i-geometric-complexity/chapter-2-terrain-rendering-using-gpu-based-geometry) | Incremental nested terrain levels; heightfield method needs adaptation for voxel caves |
| S06 | [wgpu](https://wgpu.rs/) and [architecture](https://wgpu.rs/doc/wgpu/documentation/internals/architecture/index.html) | Safe API and underlying validation/backend boundary; not a renderer |
| S07 | [Iris shader introduction](https://shaders.properties/current/guides/your-first-shaderpack/0_intro/) | Existing packs use GLSL/OpenGL conventions |
| S08 | [Iris program stages](https://shaders.properties/current/reference/programs/overview/) and [settings](https://shaders.properties/current/reference/shadersproperties/overview/) | Compatibility extends beyond source-language translation |
| S09 | [DeepSeek thinking](https://api-docs.deepseek.com/guides/thinking_mode/) and [API](https://api-docs.deepseek.com/api/create-chat-completion/) | Actual model and parameter support; pages have effort-field ambiguity, verify integration |
| S10 | [Cline DeepSeek](https://docs.cline.bot/provider-config/deepseek) | Provider/key/model setup; installed support needs smoke test |
| S11 | [Cline rules](https://docs.cline.bot/customization/cline-rules) | Markdown workspace rules and AGENTS integration |
| S12 | [GitHub Wiki](https://docs.github.com/en/communities/documenting-your-project-with-wikis/adding-or-editing-wiki-pages) | Separate Wiki Git repository and initial-page setup |
| S13 | [Minecraft EULA](https://www.minecraft.net/en-us/eula) and [usage guidelines](https://www.minecraft.net/en-us/usage-guidelines) | Distribution/branding restrictions; local extraction is not blanket legal clearance |
| S14 | [Horizon API overview](https://playhorizon.dev/docs/apis) | Explicit API maturity/versioning; project claims not independent validation |
| S15 | [Horizon build approach](https://playhorizon.dev/docs/how-horizon-is-built) | Provenance and test philosophy; do not copy implementation |
| S16 | [Data generators](https://minecraft.wiki/w/Tutorial:Running_the_data_generator) | Community description of the vanilla server `--reports` generator used for inventory; added 10 November 2026, verify the invocation against the pinned server jar |

## Horizon inspiration, independently applied

Take the idea of driving reference and reimplementation with the same inputs, separating integrated-server authority, and exposing documented automation and extension contracts. Treat local resource preparation and honest compatibility reporting as useful architectural lessons. Sunset's implementations, ABI, renderer and fixtures remain original.

Do not copy Horizon source, shader loader, hook layout or headline performance claims. Its published harness counts differ by page and date and are self-reported; this handbook deliberately does not adopt them as proof or a completion target. Sunset needs its own inventory and independent results.

## Unverified matters

Owner hardware, original ~3,500 FPS settings and baseline mod list; wgpu multi-draw-indirect and Immediate present support per backend in the pinned version; exact target artifacts/protocol/pack metadata; full GLSL compatibility feasibility; chosen Wasm runtime cost; installed Cline/V4 integration; current Wiki write permissions. Resolve through milestone evidence, not assumptions.
