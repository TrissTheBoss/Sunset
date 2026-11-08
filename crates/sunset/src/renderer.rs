//! Chunk meshing and block targeting. Geometry is derived from the same
//! block/chunk representation that gameplay uses (PRD 3.1, 4.6).

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use sunset_core::block::{block_def, is_transparent, occludes, BlockState, Shape, AIR};
use sunset_core::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use sunset_core::world::World as GameWorld;

use crate::mcassets::FaceTextures;
use crate::Session;

/// The block currently under the crosshair (and the face that was hit).
#[derive(Resource, Default)]
pub struct Target(pub Option<TargetHit>);

#[derive(Clone, Copy)]
pub struct TargetHit {
    pub pos: [i32; 3],
    pub face: [i32; 3],
}

/// The target texture atlas: per-block face textures, a texture-name → UV-rect
/// map, and the atlas image handle. `None` means assets were not available, so
/// the renderer falls back to flat authored colors.
#[derive(Resource)]
pub struct BlockAtlas {
    pub face_textures: HashMap<BlockState, FaceTextures>,
    pub uv: HashMap<String, [f32; 4]>,
    pub handle: Handle<Image>,
}

/// Marker for the combined world mesh, carrying the total chunk revision it was
/// built from and which pass (opaque vs transparent) it belongs to.
#[derive(Component)]
pub struct WorldMesh {
    pub revision: u64,
    pub transparent: bool,
}

/// The four corners of face `face` for an axis-aligned box `[min, max]`.
fn face_corners(min: Vec3, max: Vec3, face: usize) -> [[f32; 3]; 4] {
    match face {
        0 => [
            [min.x, max.y, min.z],
            [max.x, max.y, min.z],
            [max.x, max.y, max.z],
            [min.x, max.y, max.z],
        ],
        1 => [
            [min.x, min.y, min.z],
            [max.x, min.y, min.z],
            [max.x, min.y, max.z],
            [min.x, min.y, max.z],
        ],
        2 => [
            [max.x, min.y, min.z],
            [max.x, max.y, min.z],
            [max.x, max.y, max.z],
            [max.x, min.y, max.z],
        ],
        3 => [
            [min.x, min.y, min.z],
            [min.x, min.y, max.z],
            [min.x, max.y, max.z],
            [min.x, max.y, min.z],
        ],
        4 => [
            [min.x, min.y, max.z],
            [max.x, min.y, max.z],
            [max.x, max.y, max.z],
            [min.x, max.y, max.z],
        ],
        5 => [
            [min.x, min.y, min.z],
            [min.x, max.y, min.z],
            [max.x, max.y, min.z],
            [max.x, min.y, min.z],
        ],
        _ => unreachable!(),
    }
}

/// An in-progress mesh (opaque or transparent), accumulated as vertex data.
struct MeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl MeshData {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_face(
        &mut self,
        corners: [[f32; 3]; 4],
        uvs: [[f32; 2]; 4],
        normal: [f32; 3],
        color: [f32; 4],
    ) {
        let base = self.positions.len() as u32;
        for i in 0..4 {
            self.positions.push(corners[i]);
            self.normals.push(normal);
            self.colors.push(color);
            self.uvs.push(uvs[i]);
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Append another mesh's data, offsetting positions by `(ox, 0, oz)`. This
    /// lets per-chunk meshes be combined into one world mesh so the renderer
    /// issues a single draw call per pass instead of one per chunk.
    fn merge(&mut self, other: MeshData, ox: f32, oz: f32) {
        let base = self.positions.len() as u32;
        for (i, p) in other.positions.iter().enumerate() {
            self.positions.push([p[0] + ox, p[1], p[2] + oz]);
            self.normals.push(other.normals[i]);
            self.colors.push(other.colors[i]);
            self.uvs.push(other.uvs[i]);
        }
        for idx in other.indices {
            self.indices.push(base + idx);
        }
    }

    fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

/// Neighbor offsets for each face (indexed like `face_corners`).
const FACE_NORMALS: [[i32; 3]; 6] = [
    [0, 1, 0],
    [0, -1, 0],
    [1, 0, 0],
    [-1, 0, 0],
    [0, 0, 1],
    [0, 0, -1],
];

/// Authored fixed per-face brightness (Minecraft-like face shading).
const FACE_SHADE: [f32; 6] = [1.0, 0.5, 0.6, 0.6, 0.8, 0.8];

/// Texture-space UVs per face corner (0..1), indexed like `face_corners`, so
/// textures render upright.
const FACE_UVS: [[[f32; 2]; 4]; 6] = [
    // +Y (top)
    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
    // -Y (bottom)
    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
    // +X (east)
    [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
    // -X (west)
    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    // +Z (south)
    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    // -Z (north)
    [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
];

/// For each face: `(normal axis, tangent1 axis, tangent2 axis)` using
/// `x=0, y=1, z=2`. `tangent1` maps to the texture's horizontal (u) direction
/// and `tangent2` to the vertical (v) direction, matching `FACE_UVS`.
const FACE_AXES: [(usize, usize, usize); 6] = [
    (1, 0, 2), // +Y: normal Y, tangents X and Z
    (1, 0, 2), // -Y
    (0, 2, 1), // +X: normal X, tangents Z and Y
    (0, 2, 1), // -X
    (2, 0, 1), // +Z: normal Z, tangents X and Y
    (2, 0, 1), // -Z
];

/// Whether a face of `state` (whose neighbor in that face's direction is
/// `neighbor`) should be drawn.
fn face_visible(state: BlockState, neighbor: BlockState) -> bool {
    if neighbor == AIR {
        return true;
    }
    if is_transparent(state) {
        // Transparent faces are hidden against other transparent blocks
        // (glass-glass), but stay visible against opaque blocks.
        !is_transparent(neighbor)
    } else {
        // Opaque faces are hidden against opaque full cubes.
        !occludes(neighbor)
    }
}

pub fn spawn_chunk_meshes(
    mut commands: Commands,
    session: Res<Session>,
    atlas: Option<Res<BlockAtlas>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let texture: Option<Handle<Image>> = atlas.as_ref().map(|a| a.handle.clone());
    // Two shared unlit materials: opaque and transparent (alpha-blended).
    let opaque_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: texture.clone(),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let transparent_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: texture,
        unlit: true,
        cull_mode: None,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Combine all chunks into one mesh per pass (a single draw call each).
    let (opaque, transparent, revision) = build_world_meshes(&session.0.world, atlas.as_deref());
    commands.spawn((
        Mesh3d(meshes.add(opaque.build())),
        MeshMaterial3d(opaque_mat),
        WorldMesh {
            revision,
            transparent: false,
        },
    ));
    if !transparent.is_empty() {
        commands.spawn((
            Mesh3d(meshes.add(transparent.build())),
            MeshMaterial3d(transparent_mat),
            WorldMesh {
                revision,
                transparent: true,
            },
        ));
    }
}

/// Build the combined opaque and transparent meshes for the whole world, along
/// with the summed chunk revision used for change detection.
fn build_world_meshes(
    world: &GameWorld,
    atlas: Option<&BlockAtlas>,
) -> (MeshData, MeshData, u64) {
    let mut opaque = MeshData::new();
    let mut transparent = MeshData::new();
    let mut revision = 0u64;
    for chunk in world.chunks() {
        let (o, t) = build_chunk_meshes(world, chunk, atlas);
        let ox = (chunk.cx * CHUNK_SIZE as i32) as f32;
        let oz = (chunk.cz * CHUNK_SIZE as i32) as f32;
        opaque.merge(o, ox, oz);
        transparent.merge(t, ox, oz);
        revision = revision.wrapping_add(chunk.revision);
    }
    (opaque, transparent, revision)
}

/// Re-mesh the combined world mesh when any chunk's revision has changed.
pub fn remesh_system(
    session: Res<Session>,
    atlas: Option<Res<BlockAtlas>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut WorldMesh, &mut Mesh3d)>,
) {
    let world = &session.0.world;
    let mut revision = 0u64;
    for chunk in world.chunks() {
        revision = revision.wrapping_add(chunk.revision);
    }
    if query.iter().all(|(wm, _)| wm.revision == revision) {
        return;
    }

    let (opaque, transparent, _) = build_world_meshes(world, atlas.as_deref());
    let opaque_handle = meshes.add(opaque.build());
    let transparent_handle = meshes.add(transparent.build());
    for (mut wm, mut mesh3d) in &mut query {
        *mesh3d = Mesh3d(if wm.transparent {
            transparent_handle.clone()
        } else {
            opaque_handle.clone()
        });
        wm.revision = revision;
    }
}

/// Draw a wireframe selection outline around the targeted block, sized to the
/// block's actual collision shape (so slabs outline their half-height box).
pub fn outline_system(target: Res<Target>, session: Res<Session>, mut gizmos: Gizmos) {
    if let Some(hit) = &target.0 {
        let state = session.0.world.get_block(hit.pos[0], hit.pos[1], hit.pos[2]);
        if let Some((bmin, bmax)) = block_def(state).collision_box() {
            let base = Vec3::new(hit.pos[0] as f32, hit.pos[1] as f32, hit.pos[2] as f32);
            draw_box(&mut gizmos, base + bmin, base + bmax, Color::BLACK);
        }
    }
}

fn draw_box(gizmos: &mut Gizmos, min: Vec3, max: Vec3, color: Color) {
    let c = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ];
    let edges = [
        (0, 1),
        (0, 2),
        (1, 3),
        (2, 3), // -Z face
        (4, 5),
        (4, 6),
        (5, 7),
        (6, 7), // +Z face
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7), // connecting
    ];
    for (a, b) in edges {
        gizmos.line(c[a], c[b], color);
    }
}

/// Greedy-mesh one face direction for full-cube blocks: sweep each plane
/// perpendicular to the face normal and merge runs of coplanar same-material
/// faces into maximal rectangles, collapsing large flat surfaces to one quad
/// per run instead of one quad per block.
#[allow(clippy::too_many_arguments)]
fn greedy_face(
    world: &GameWorld,
    chunk: &Chunk,
    atlas: Option<&BlockAtlas>,
    base_x: i32,
    base_z: i32,
    face: usize,
    opaque: &mut MeshData,
    transparent: &mut MeshData,
) {
    let [nx, ny, nz] = FACE_NORMALS[face];
    let shade = FACE_SHADE[face];
    let (na, ta, tb) = FACE_AXES[face];
    let n_dim = if na == 1 { CHUNK_HEIGHT } else { CHUNK_SIZE };
    let a_dim = if ta == 1 { CHUNK_HEIGHT } else { CHUNK_SIZE };
    let b_dim = if tb == 1 { CHUNK_HEIGHT } else { CHUNK_SIZE };

    for plane in 0..n_dim {
        let mut mask: Vec<Option<(BlockState, Option<[f32; 4]>)>> = vec![None; a_dim * b_dim];
        for a in 0..a_dim {
            for b in 0..b_dim {
                let mut c = [0usize; 3];
                c[na] = plane;
                c[ta] = a;
                c[tb] = b;
                let (x, y, z) = (c[0], c[1], c[2]);
                let state = chunk.get(x, y, z);
                if state == AIR {
                    continue;
                }
                let def = block_def(state);
                if def.shape != Shape::FullCube {
                    continue;
                }
                let neighbor = world.get_block(
                    base_x + x as i32 + nx,
                    y as i32 + ny,
                    base_z + z as i32 + nz,
                );
                if !face_visible(state, neighbor) {
                    continue;
                }
                let rect = atlas
                    .and_then(|at| at.face_textures.get(&state))
                    .and_then(|ft| ft.texture_for_face([nx, ny, nz], def.end_axis))
                    .and_then(|tex| atlas.and_then(|at| at.uv.get(tex)).copied());
                mask[a * b_dim + b] = Some((state, rect));
            }
        }

        let mut visited = vec![false; a_dim * b_dim];
        for a in 0..a_dim {
            for b in 0..b_dim {
                let idx = a * b_dim + b;
                if visited[idx] {
                    continue;
                }
                let Some(cell) = mask[idx] else {
                    continue;
                };
                let (state, rect) = cell;
                let mut w = 1;
                while a + w < a_dim
                    && !visited[(a + w) * b_dim + b]
                    && mask[(a + w) * b_dim + b] == Some(cell)
                {
                    w += 1;
                }
                let mut h = 1;
                'grow: while b + h < b_dim {
                    for da in 0..w {
                        if visited[(a + da) * b_dim + (b + h)]
                            || mask[(a + da) * b_dim + (b + h)] != Some(cell)
                        {
                            break 'grow;
                        }
                    }
                    h += 1;
                }
                for da in 0..w {
                    for db in 0..h {
                        visited[(a + da) * b_dim + (b + db)] = true;
                    }
                }
                let def = block_def(state);
                let target = if is_transparent(state) {
                    &mut *transparent
                } else {
                    &mut *opaque
                };
                let normal = [nx as f32, ny as f32, nz as f32];
                let dir = FACE_NORMALS[face][na];
                let mut mn = [0.0f32; 3];
                let mut mx = [0.0f32; 3];
                let n_coord = plane as f32 + if dir > 0 { 1.0 } else { 0.0 };
                mn[na] = n_coord;
                mx[na] = n_coord;
                mn[ta] = a as f32;
                mx[ta] = (a + w) as f32;
                mn[tb] = b as f32;
                mx[tb] = (b + h) as f32;
                let corners = face_corners(Vec3::from_array(mn), Vec3::from_array(mx), face);
                if let Some(rect) = rect {
                    let uvs = [0usize, 1, 2, 3].map(|i| {
                        let [u, v] = FACE_UVS[face][i];
                        [
                            rect[0] + u * w as f32 * (rect[2] - rect[0]),
                            rect[1] + v * h as f32 * (rect[3] - rect[1]),
                        ]
                    });
                    target.add_face(corners, uvs, normal, [shade, shade, shade, 1.0]);
                } else {
                    let fc = def.face_color(FACE_NORMALS[face]);
                    let alpha = if is_transparent(state) { 0.45 } else { 1.0 };
                    target.add_face(
                        corners,
                        [[0.0, 0.0]; 4],
                        normal,
                        [
                            fc[0] as f32 / 255.0 * shade,
                            fc[1] as f32 / 255.0 * shade,
                            fc[2] as f32 / 255.0 * shade,
                            alpha,
                        ],
                    );
                }
            }
        }
    }
}


/// Build the opaque and transparent meshes for one chunk, emitting only faces
/// that border a non-occluding block (including across chunk boundaries). When
/// a target texture atlas is available, faces are textured; otherwise they fall
/// back to the authored flat color.
fn build_chunk_meshes(
    world: &GameWorld,
    chunk: &Chunk,
    atlas: Option<&BlockAtlas>,
) -> (MeshData, MeshData) {
    let mut opaque = MeshData::new();
    let mut transparent = MeshData::new();

    let base_x = chunk.cx * CHUNK_SIZE as i32;
    let base_z = chunk.cz * CHUNK_SIZE as i32;

    // Greedy-mesh full cubes first (the bulk of the geometry).
    for face in 0..6 {
        greedy_face(world, chunk, atlas, base_x, base_z, face, &mut opaque, &mut transparent);
    }

    // Non-full-cube blocks (slabs) are meshed per-face below; their faces are
    // shorter than a block, so they can't be merged safely.
    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let state = chunk.get(x, y, z);
                if state == AIR {
                    continue;
                }
                let def = block_def(state);
                // Full cubes are already greedy-meshed above.
                if def.shape == Shape::FullCube {
                    continue;
                }
                let Some((bmin, bmax)) = def.collision_box() else {
                    continue;
                };
                let wx = base_x + x as i32;
                let wz = base_z + z as i32;
                let wy = y as i32;
                let target = if is_transparent(state) {
                    &mut transparent
                } else {
                    &mut opaque
                };

                for face in 0..6 {
                    let [nx, ny, nz] = FACE_NORMALS[face];
                    let neighbor = world.get_block(wx + nx, wy + ny, wz + nz);
                    if neighbor != AIR {
                        if is_transparent(state) {
                            // Transparent faces are hidden against other
                            // transparent blocks (glass-glass), but stay visible
                            // against air and opaque blocks.
                            if is_transparent(neighbor) {
                                continue;
                            }
                        } else if occludes(neighbor) {
                            continue;
                        }
                    }
                    let shade = FACE_SHADE[face];
                    let corners = face_corners(
                        Vec3::new(x as f32 + bmin.x, y as f32 + bmin.y, z as f32 + bmin.z),
                        Vec3::new(x as f32 + bmax.x, y as f32 + bmax.y, z as f32 + bmax.z),
                        face,
                    );
                    let normal = [nx as f32, ny as f32, nz as f32];

                    // Prefer the target texture when available.
                    let mut textured = false;
                    if let Some(atlas) = atlas {
                        if let Some(ft) = atlas.face_textures.get(&state) {
                            if let Some(tex) = ft.texture_for_face([nx, ny, nz], def.end_axis) {
                                if let Some(&rect) = atlas.uv.get(tex) {
                                    let uvs = FACE_UVS[face].map(|[u, v]| {
                                        // Slab side faces use only half of the
                                        // `*_slab_side` texture (it packs both
                                        // the top and bottom slab halves).
                                        let v = match def.shape {
                                            Shape::SlabBottom if face >= 2 => 0.5 + 0.5 * v,
                                            Shape::SlabTop if face >= 2 => 0.5 * v,
                                            _ => v,
                                        };
                                        [
                                            rect[0] + u * (rect[2] - rect[0]),
                                            rect[1] + v * (rect[3] - rect[1]),
                                        ]
                                    });
                                    let color = [shade, shade, shade, 1.0];
                                    target.add_face(corners, uvs, normal, color);
                                    textured = true;
                                }
                            }
                        }
                    }

                    if !textured {
                        // Fallback: authored flat color.
                        let fc = def.face_color([nx, ny, nz]);
                        let alpha = if is_transparent(state) { 0.45 } else { 1.0 };
                        let color: [f32; 4] = [
                            fc[0] as f32 / 255.0 * shade,
                            fc[1] as f32 / 255.0 * shade,
                            fc[2] as f32 / 255.0 * shade,
                            alpha,
                        ];
                        target.add_face(corners, [[0.0, 0.0]; 4], normal, color);
                    }
                }
            }
        }
    }

    (opaque, transparent)
}
