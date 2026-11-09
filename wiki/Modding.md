# Modding Sunset

Planned platform: Sunset Modules. Choose data definitions for content, sandboxed Wasm behavior for controlled logic, or a trusted source-build SDK for engine replacements. APIs are not yet implemented or stable.

Every module uses a namespaced ID and versioned dependencies/capabilities. Validate definitions before activation. Behavior uses ordered batched events and bounded command buffers. Do not cross the host boundary for each voxel. Trusted source modules have application privileges and are not sandboxed.

Coverage includes blocks/states/items/entities/AI/particles/materials/worldgen/commands/menus/audio/network/save systems and core adapters. Each API page must eventually state lifecycle, ownership, thread, order, limit, version and example with a passing contract test.

Vanilla-server mode permits only compatible local behavior. Gameplay-changing modules use separate profiles and compatible peers. Existing Java mod jars are not supported binaries.

Before stable API release, publish complete schema/reference/tutorial/migration/troubleshooting pages generated from actual contracts. Do not invent callable methods in documentation.
