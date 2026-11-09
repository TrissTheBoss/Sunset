//! Greedy chunk meshing. Emits raw interleaved vertices + indices with chunk
//! offsets baked into world-space positions, so the renderer issues a single
//! uniform (no per-chunk transform).

use std::collections::HashMap;

use glam::Vec3;

use sunset_core::block::{block_def, is_transparent, occludes, BlockState, Shape, AIR};
use sunset_core::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use sunset_core::world::World;

use crate::assets::FaceTextures;

/// A single interleaved vertex: position, tiled UV, face shade/color, and the
/// texture's atlas rect (wrapped by `fract` in the fragment shader so merged
/// rectangles tile instead of stretch).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub rect: [f32; 4],
}

/// An in-progress mesh (opaque or transparent).
#[derive(Debug, PartialEq)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_face(
        &mut self,
        corners: [[f32; 3]; 4],
        uvs: [[f32; 2]; 4],
        color: [f32; 4],
        rect: [f32; 4],
    ) {
        let base = self.vertices.len() as u32;
        for i in 0..4 {
            self.vertices.push(Vertex {
                position: corners[i],
                uv: uvs[i],
                color,
                rect,
            });
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
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

/// Per-face brightness (Minecraft-like face shading).
const FACE_SHADE: [f32; 6] = [1.0, 0.5, 0.6, 0.6, 0.8, 0.8];

/// Texture-space UVs per face corner (0..1), indexed like `face_corners`.
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

/// For each face: (normal axis, tangent1 axis, tangent2 axis) using
/// x=0, y=1, z=2. `tangent1` maps to texture u; `tangent2` to texture v.
const FACE_AXES: [(usize, usize, usize); 6] = [
    (1, 0, 2),
    (1, 0, 2),
    (0, 2, 1),
    (0, 2, 1),
    (2, 0, 1),
    (2, 0, 1),
];

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

/// Whether a face of `state` (bordered by `neighbor`) should be drawn.
fn face_visible(state: BlockState, neighbor: BlockState) -> bool {
    if neighbor == AIR {
        return true;
    }
    if is_transparent(state) {
        !is_transparent(neighbor)
    } else {
        !occludes(neighbor)
    }
}

/// Greedy-mesh one face direction for full-cube blocks, merging coplanar
/// same-material faces into maximal rectangles.
#[allow(clippy::too_many_arguments)]
fn greedy_face(
    world: &World,
    chunk: &Chunk,
    face_textures: &HashMap<BlockState, FaceTextures>,
    uv: &HashMap<String, [f32; 4]>,
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
                let rect = face_textures
                    .get(&state)
                    .and_then(|ft| ft.texture_for_face([nx, ny, nz], def.end_axis))
                    .and_then(|tex| uv.get(tex).copied());
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
                // Bake the chunk's world offset into the positions.
                mn[0] += base_x as f32;
                mn[2] += base_z as f32;
                mx[0] += base_x as f32;
                mx[2] += base_z as f32;
                let corners = face_corners(Vec3::from_array(mn), Vec3::from_array(mx), face);

                if let Some(rect) = rect {
                    // Tiled UV (w x h tiles); the shader wraps it into `rect`.
                    let uvs = [0usize, 1, 2, 3].map(|i| {
                        let [u, v] = FACE_UVS[face][i];
                        [u * w as f32, v * h as f32]
                    });
                    target.add_face(corners, uvs, [shade, shade, shade, 1.0], rect);
                } else {
                    let fc = def.face_color(FACE_NORMALS[face]);
                    let alpha = if is_transparent(state) { 0.45 } else { 1.0 };
                    target.add_face(
                        corners,
                        [[0.0, 0.0]; 4],
                        [
                            fc[0] as f32 / 255.0 * shade,
                            fc[1] as f32 / 255.0 * shade,
                            fc[2] as f32 / 255.0 * shade,
                            alpha,
                        ],
                        [0.0, 0.0, 1.0, 1.0],
                    );
                }
            }
        }
    }
}

/// Build the opaque and transparent meshes for one chunk (world-space positions).
pub fn build_chunk_meshes(
    world: &World,
    chunk: &Chunk,
    face_textures: &HashMap<BlockState, FaceTextures>,
    uv: &HashMap<String, [f32; 4]>,
) -> (MeshData, MeshData) {
    let mut opaque = MeshData::new();
    let mut transparent = MeshData::new();

    let base_x = chunk.cx * CHUNK_SIZE as i32;
    let base_z = chunk.cz * CHUNK_SIZE as i32;

    // Greedy-mesh full cubes.
    for face in 0..6 {
        greedy_face(
            world,
            chunk,
            face_textures,
            uv,
            base_x,
            base_z,
            face,
            &mut opaque,
            &mut transparent,
        );
    }

    // Non-full-cube blocks (slabs) are meshed per-face.
    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let state = chunk.get(x, y, z);
                if state == AIR {
                    continue;
                }
                let def = block_def(state);
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
                    if !face_visible(state, neighbor) {
                        continue;
                    }
                    let shade = FACE_SHADE[face];
                    let corners = face_corners(
                        Vec3::new(
                            x as f32 + bmin.x + base_x as f32,
                            y as f32 + bmin.y,
                            z as f32 + bmin.z + base_z as f32,
                        ),
                        Vec3::new(
                            x as f32 + bmax.x + base_x as f32,
                            y as f32 + bmax.y,
                            z as f32 + bmax.z + base_z as f32,
                        ),
                        face,
                    );

                    let mut textured = false;
                    if let Some(ft) = face_textures.get(&state) {
                        if let Some(tex) = ft.texture_for_face([nx, ny, nz], def.end_axis) {
                            if let Some(&rect) = uv.get(tex) {
                                let uvs = FACE_UVS[face].map(|[u, v]| {
                                    let v = match def.shape {
                                        Shape::SlabBottom if face >= 2 => 0.5 + 0.5 * v,
                                        Shape::SlabTop if face >= 2 => 0.5 * v,
                                        _ => v,
                                    };
                                    [u, v]
                                });
                                target.add_face(corners, uvs, [shade, shade, shade, 1.0], rect);
                                textured = true;
                            }
                        }
                    }
                    if !textured {
                        let fc = def.face_color([nx, ny, nz]);
                        let alpha = if is_transparent(state) { 0.45 } else { 1.0 };
                        target.add_face(
                            corners,
                            [[0.0, 0.0]; 4],
                            [
                                fc[0] as f32 / 255.0 * shade,
                                fc[1] as f32 / 255.0 * shade,
                                fc[2] as f32 / 255.0 * shade,
                                alpha,
                            ],
                            [0.0, 0.0, 1.0, 1.0],
                        );
                    }
                }
            }
        }
    }


    (opaque, transparent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use sunset_core::fixture;

    /// A boundary edit (changing a block on a chunk edge) must produce, after
    /// re-meshing only the affected chunk and its four neighbours, exactly the
    /// same meshes as a full rebuild of every chunk.
    #[test]
    fn boundary_edit_matches_full_rebuild() {
        let face_textures: HashMap<BlockState, FaceTextures> = HashMap::new();
        let uv: HashMap<String, [f32; 4]> = HashMap::new();

        let mut world = fixture::building_yard();

        // Pre-edit meshes for every chunk (the "incremental" starting point).
        let mut incremental: HashMap<(i32, i32), (MeshData, MeshData)> = HashMap::new();
        for chunk in world.chunks() {
            incremental.insert(
                (chunk.cx, chunk.cz),
                build_chunk_meshes(&world, chunk, &face_textures, &uv),
            );
        }

        // Break a block on a chunk boundary: the cobblestone wall at x=0 is the
        // left edge of chunk cx=0 (adjacent to chunk cx=-1 across the seam).
        let (wx, wy, wz): (i32, i32, i32) = (0, 7, 0);
        let edited = (
            wx.div_euclid(CHUNK_SIZE as i32),
            wz.div_euclid(CHUNK_SIZE as i32),
        );
        world.set_block(wx, wy, wz, AIR);

        // Re-mesh only the edited chunk + its four neighbours.
        let mut dirty = vec![
            edited,
            (edited.0 + 1, edited.1),
            (edited.0 - 1, edited.1),
            (edited.0, edited.1 + 1),
            (edited.0, edited.1 - 1),
        ];
        dirty.sort_unstable();
        dirty.dedup();
        for (cx, cz) in dirty {
            if let Some(chunk) = world.get_chunk(cx, cz) {
                incremental.insert(
                    (cx, cz),
                    build_chunk_meshes(&world, chunk, &face_textures, &uv),
                );
            }
        }

        // Full rebuild reference after the edit.
        let mut full: HashMap<(i32, i32), (MeshData, MeshData)> = HashMap::new();
        for chunk in world.chunks() {
            full.insert(
                (chunk.cx, chunk.cz),
                build_chunk_meshes(&world, chunk, &face_textures, &uv),
            );
        }

        assert_eq!(incremental.len(), full.len(), "chunk set must match");
        for (key, (o, t)) in &full {
            let (io, it) = &incremental[key];
            assert_eq!(io, o, "opaque mesh mismatch for chunk {key:?}");
            assert_eq!(it, t, "transparent mesh mismatch for chunk {key:?}");
        }
    }
}

