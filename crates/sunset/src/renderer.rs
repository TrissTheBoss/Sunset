//! Chunk meshing and block targeting. Geometry is derived from the same
//! block/chunk representation that gameplay uses (PRD 3.1, 4.6).

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use sunset_core::block::{block_def, is_transparent, occludes, BlockState, AIR};
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

/// Marker for a spawned chunk mesh, carrying identity, revision, and which pass
/// (opaque vs transparent) it belongs to, so edits can trigger a targeted
/// re-mesh (PRD 4.6).
#[derive(Component)]
pub struct ChunkMesh {
    pub cx: i32,
    pub cz: i32,
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

    let world = &session.0.world;
    for chunk in world.chunks() {
        let (opaque, transparent) = build_chunk_meshes(world, chunk, atlas.as_deref());
        let ox = (chunk.cx * CHUNK_SIZE as i32) as f32;
        let oz = (chunk.cz * CHUNK_SIZE as i32) as f32;
        let tf = Transform::from_xyz(ox, 0.0, oz);
        commands.spawn((
            Mesh3d(meshes.add(opaque.build())),
            MeshMaterial3d(opaque_mat.clone()),
            tf,
            ChunkMesh {
                cx: chunk.cx,
                cz: chunk.cz,
                revision: chunk.revision,
                transparent: false,
            },
        ));
        if !transparent.is_empty() {
            commands.spawn((
                Mesh3d(meshes.add(transparent.build())),
                MeshMaterial3d(transparent_mat.clone()),
                tf,
                ChunkMesh {
                    cx: chunk.cx,
                    cz: chunk.cz,
                    revision: chunk.revision,
                    transparent: true,
                },
            ));
        }
    }
}

/// Re-mesh any chunk whose authoritative revision has changed since it was last
/// built (block edits).
pub fn remesh_system(
    session: Res<Session>,
    atlas: Option<Res<BlockAtlas>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut ChunkMesh, &mut Mesh3d)>,
) {
    let world = &session.0.world;
    for (mut cm, mut mesh3d) in &mut query {
        if let Some(chunk) = world.get_chunk(cm.cx, cm.cz) {
            if chunk.revision != cm.revision {
                let (opaque, transparent) = build_chunk_meshes(world, chunk, atlas.as_deref());
                let data = if cm.transparent { transparent } else { opaque };
                *mesh3d = Mesh3d(meshes.add(data.build()));
                cm.revision = chunk.revision;
            }
        }
    }
}

/// Draw a wireframe selection outline around the targeted block.
pub fn outline_system(target: Res<Target>, mut gizmos: Gizmos) {
    if let Some(hit) = &target.0 {
        let min = Vec3::new(hit.pos[0] as f32, hit.pos[1] as f32, hit.pos[2] as f32);
        let max = min + Vec3::ONE;
        draw_box(&mut gizmos, min, max, Color::BLACK);
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

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let state = chunk.get(x, y, z);
                if state == AIR {
                    continue;
                }
                let def = block_def(state);
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
                    if occludes(world.get_block(wx + nx, wy + ny, wz + nz)) {
                        continue;
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
