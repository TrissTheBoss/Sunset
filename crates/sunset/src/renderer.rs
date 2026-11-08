//! Chunk meshing: turns the authoritative world into render meshes.
//!
//! Geometry is derived from the same block/chunk representation that gameplay
//! uses (PRD 3.1). A disconnected or pre-rendered scene would not satisfy M0.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use sunset_core::block::{block_def, is_transparent};
use sunset_core::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use sunset_core::world::World as GameWorld;

/// Marker for a spawned chunk mesh, carrying identity and revision for the
/// dynamic re-meshing introduced in later milestones (PRD 4.6).
#[derive(Component)]
#[allow(dead_code)] // fields are read by later milestones' re-meshing
pub struct ChunkMesh {
    pub cx: i32,
    pub cz: i32,
    pub revision: u64,
}

/// Unit-cube face corners, one entry per face, four corners each.
const FACES: [[[f32; 3]; 4]; 6] = [
    // +Y (top)
    [[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
    // -Y (bottom)
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    // +X (east)
    [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [1.0, 0.0, 1.0]],
    // -X (west)
    [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]],
    // +Z (south)
    [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
    // -Z (north)
    [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
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
    world: Res<super::CoreWorld>,
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

    let world_ref: &GameWorld = &world.0;
    for chunk in world_ref.chunks() {
        let handle = meshes.add(build_chunk_mesh(world_ref, chunk));
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
