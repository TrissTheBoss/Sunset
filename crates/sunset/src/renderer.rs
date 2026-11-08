//! Chunk meshing and block targeting. Geometry is derived from the same
//! block/chunk representation that gameplay uses (PRD 3.1, 4.6).

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use sunset_core::block::{block_def, is_transparent};
use sunset_core::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use sunset_core::world::World as GameWorld;

use crate::Session;

/// The block currently under the crosshair (and the face that was hit).
#[derive(Resource, Default)]
pub struct Target(pub Option<TargetHit>);

#[derive(Clone, Copy)]
pub struct TargetHit {
    pub pos: [i32; 3],
    pub face: [i32; 3],
}

/// Marker for a spawned chunk mesh, carrying identity and revision so edits can
/// trigger a targeted re-mesh (PRD 4.6).
#[derive(Component)]
pub struct ChunkMesh {
    pub cx: i32,
    pub cz: i32,
    pub revision: u64,
}

/// Unit-cube face corners, one entry per face, four corners each.
const FACES: [[[f32; 3]; 4]; 6] = [
    // +Y (top)
    [
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ],
    // -Y (bottom)
    [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ],
    // +X (east)
    [
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [1.0, 0.0, 1.0],
    ],
    // -X (west)
    [
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 0.0],
    ],
    // +Z (south)
    [
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ],
    // -Z (north)
    [
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
    ],
];

/// Neighbor offsets indexed like [`FACES`].
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

pub fn spawn_chunk_meshes(
    mut commands: Commands,
    session: Res<Session>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Single shared unlit material: block color is baked into vertex colors.
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let world = &session.0.world;
    for chunk in world.chunks() {
        let handle = meshes.add(build_chunk_mesh(world, chunk));
        commands.spawn((
            Mesh3d(handle),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(
                (chunk.cx * CHUNK_SIZE as i32) as f32,
                0.0,
                (chunk.cz * CHUNK_SIZE as i32) as f32,
            ),
            ChunkMesh {
                cx: chunk.cx,
                cz: chunk.cz,
                revision: chunk.revision,
            },
        ));
    }
}

/// Re-mesh any chunk whose authoritative revision has changed since it was last
/// built (block edits).
pub fn remesh_system(
    session: Res<Session>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut ChunkMesh, &mut Mesh3d)>,
) {
    let world = &session.0.world;
    for (mut cm, mut mesh3d) in &mut query {
        if let Some(chunk) = world.get_chunk(cm.cx, cm.cz) {
            if chunk.revision != cm.revision {
                *mesh3d = Mesh3d(meshes.add(build_chunk_mesh(world, chunk)));
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

/// Build a mesh for one chunk, emitting only faces that border air
/// (including across chunk boundaries via world-coordinate lookup).
fn build_chunk_mesh(world: &GameWorld, chunk: &Chunk) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let base_x = chunk.cx * CHUNK_SIZE as i32;
    let base_z = chunk.cz * CHUNK_SIZE as i32;

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let state = chunk.get(x, y, z);
                if is_transparent(state) {
                    continue;
                }
                let def = block_def(state);
                let r = def.color[0] as f32 / 255.0;
                let g = def.color[1] as f32 / 255.0;
                let b = def.color[2] as f32 / 255.0;

                let wx = base_x + x as i32;
                let wz = base_z + z as i32;
                let wy = y as i32;

                for face in 0..6 {
                    let [nx, ny, nz] = FACE_NORMALS[face];
                    if !is_transparent(world.get_block(wx + nx, wy + ny, wz + nz)) {
                        continue;
                    }

                    let shade = FACE_SHADE[face];
                    let color: [f32; 4] = [r * shade, g * shade, b * shade, 1.0];
                    let base = positions.len() as u32;

                    for corner in FACES[face] {
                        positions.push([
                            x as f32 + corner[0],
                            y as f32 + corner[1],
                            z as f32 + corner[2],
                        ]);
                        normals.push([nx as f32, ny as f32, nz as f32]);
                        colors.push(color);
                    }
                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
