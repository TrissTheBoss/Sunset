//! Greedy chunk meshing. Emits raw interleaved vertices + indices with chunk
//! offsets baked into world-space positions, so the renderer issues a single
//! uniform (no per-chunk transform).

use std::collections::HashMap;

use glam::Vec3;

use sunset_core::block::{block_def, is_transparent, occludes, BlockState, Shape, AIR, BLOCKS};
use sunset_core::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use sunset_core::world::World;

use crate::assets::FaceTextures;

/// Resolved per-(block, face) render facts, precomputed once after assets load
/// (M15). Removes string-keyed texture/atlas lookups from the mesher's inner
/// loops.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceDesc {
    /// Atlas rect (normalized 0..1) when textured, else `None` (use `color`).
    pub rect: Option<[f32; 4]>,
    /// Vertex colour (face-shaded), RGBA 0..1.
    pub color: [f32; 4],
}

/// A renderer-owned face-description table indexed by block state then face.
pub struct FaceTable {
    descs: Vec<[FaceDesc; 6]>,
}

impl FaceTable {
    /// Build the table from resolved face textures and atlas UVs.
    pub fn build(
        face_textures: &HashMap<BlockState, FaceTextures>,
        uv: &HashMap<String, [f32; 4]>,
    ) -> Self {
        let mut descs = vec![[FaceDesc { rect: None, color: [0.0; 4] }; 6]; BLOCKS.len()];
        for (state, def) in BLOCKS.iter().enumerate() {
            for face in 0..6 {
                let normal = FACE_NORMALS[face];
                let shade = FACE_SHADE[face];
                let rect = face_textures
                    .get(&(state as BlockState))
                    .and_then(|ft| ft.texture_for_face(normal, def.end_axis))
                    .and_then(|tex| uv.get(tex).copied());
                let color = match rect {
                    Some(_) => [shade, shade, shade, 1.0],
                    None => {
                        let fc = def.face_color(normal);
                        let alpha = if is_transparent(state as BlockState) { 0.45 } else { 1.0 };
                        [
                            fc[0] as f32 / 255.0 * shade,
                            fc[1] as f32 / 255.0 * shade,
                            fc[2] as f32 / 255.0 * shade,
                            alpha,
                        ]
                    }
                };
                descs[state][face] = FaceDesc { rect, color };
            }
        }
        Self { descs }
    }

    #[inline]
    pub fn get(&self, state: BlockState, face: usize) -> FaceDesc {
        self.descs[state as usize][face]
    }
}

/// A chunk plus a 1-block halo of neighbouring blocks, so face-visibility
/// lookups never cross into a `World` HashMap (M16). The word-parallel mesher
/// reads occupancy from here instead of calling `World::get_block` per cell.
pub struct ChunkSnapshot {
    /// Block states, dims `(CHUNK_SIZE+2) x (CHUNK_HEIGHT+2) x (CHUNK_SIZE+2)`,
    /// indexed `[x+1][y+1][z+1]` (y-major, matching `Chunk`).
    blocks: Box<[BlockState]>,
}

impl ChunkSnapshot {
    const DIM: usize = CHUNK_SIZE + 2;
    const HEIGHT: usize = CHUNK_HEIGHT + 2;

    pub fn gather(world: &World, chunk: &Chunk) -> Self {
        let mut blocks = vec![AIR; Self::DIM * Self::DIM * Self::HEIGHT].into_boxed_slice();
        let base_x = chunk.cx * CHUNK_SIZE as i32;
        let base_z = chunk.cz * CHUNK_SIZE as i32;
        for y in -1..=CHUNK_HEIGHT as i32 {
            for z in -1..=CHUNK_SIZE as i32 {
                for x in -1..=CHUNK_SIZE as i32 {
                    let idx =
                        ((y + 1) as usize * Self::DIM + (z + 1) as usize) * Self::DIM + (x + 1) as usize;
                    blocks[idx] = world.get_block(base_x + x, y, base_z + z);
                }
            }
        }
        Self { blocks }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> BlockState {
        let idx = ((y + 1) as usize * Self::DIM + (z + 1) as usize) * Self::DIM + (x + 1) as usize;
        self.blocks[idx]
    }
}

/// A single interleaved vertex: position, tiled UV, face shade/color, and the
/// texture's atlas rect (wrapped by `fract` in the fragment shader so merged
/// rectangles tile instead of stretch).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    /// Tiled UV in fixed-point (×256); 0.5 steps (slab half-texture) stay exact.
    pub uv: [u16; 2],
    /// RGBA8-packed face shade/color.
    pub color: u32,
    /// Atlas rect in normalized fixed-point (×65535), i.e. `0..1`.
    pub rect: [u16; 4],
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
        let color = pack_color(color);
        let rect = pack_rect(rect);
        for i in 0..4 {
            self.vertices.push(Vertex {
                position: corners[i],
                uv: pack_uv(uvs[i]),
                color,
                rect,
            });
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

/// Pack a tiled UV (multiples of 0.5) into fixed-point u16 (×256).
fn pack_uv(uv: [f32; 2]) -> [u16; 2] {
    [(uv[0] * 256.0).round() as u16, (uv[1] * 256.0).round() as u16]
}

/// Pack a `0..1` RGBA color into an RGBA8 u32.
fn pack_color(c: [f32; 4]) -> u32 {
    let r = (c[0].clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (c[1].clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (c[2].clamp(0.0, 1.0) * 255.0).round() as u32;
    let a = (c[3].clamp(0.0, 1.0) * 255.0).round() as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}

/// Pack a `0..1` atlas rect into normalized fixed-point u16 (×65535).
fn pack_rect(rect: [f32; 4]) -> [u16; 4] {
    [
        (rect[0] * 65535.0).round() as u16,
        (rect[1] * 65535.0).round() as u16,
        (rect[2] * 65535.0).round() as u16,
        (rect[3] * 65535.0).round() as u16,
    ]
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
    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
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
            [min.x, max.y, max.z],
            [max.x, max.y, max.z],
            [max.x, max.y, min.z],
            [min.x, max.y, min.z],
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

/// Whether a face of `state` (a box `self_min..self_max` facing `normal`,
/// bordered by `neighbor`) should be drawn. A neighbour suppresses the face only
/// when its own geometry actually reaches the face's surface (relevant for
/// slabs, whose faces sit inside the cell) and only per the transparency rule.
fn face_visible(
    state: BlockState,
    neighbor: BlockState,
    normal: [i32; 3],
    self_min: Vec3,
    self_max: Vec3,
) -> bool {
    if neighbor == AIR {
        return true;
    }
    // The face's surface plane along the normal axis (block-local).
    let axis = if normal[0] != 0 {
        0
    } else if normal[1] != 0 {
        1
    } else {
        2
    };
    let surface = if normal[axis] > 0 {
        self_max[axis]
    } else {
        self_min[axis]
    };
    // The neighbour covers this face only if its geometry actually reaches that
    // plane. Collision boxes use exact halves (0.0 / 0.5 / 1.0), so equality is
    // exact, not approximate.
    if let Some((nbmin, nbmax)) = block_def(neighbor).collision_box() {
        let neighbor_edge = if normal[axis] > 0 {
            1.0 + nbmin[axis]
        } else {
            -1.0 + nbmax[axis]
        };
        if neighbor_edge != surface {
            return true; // gap: the neighbour's geometry doesn't touch this face
        }
    } else {
        return true; // neighbour has no geometry
    }
    if is_transparent(state) {
        // Glass: a full-cube neighbour (opaque solid or glass) fully covers this
        // face and hides it. A slab only partially covers it (in the vertical
        // tangent direction), so the face stays visible.
        block_def(neighbor).shape != Shape::FullCube
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
    face_table: &FaceTable,
    base_x: i32,
    base_z: i32,
    face: usize,
    opaque: &mut MeshData,
    transparent: &mut MeshData,
) {
    let [nx, ny, nz] = FACE_NORMALS[face];
    let (na, ta, tb) = FACE_AXES[face];
    let n_dim = if na == 1 { CHUNK_HEIGHT } else { CHUNK_SIZE };
    let a_dim = if ta == 1 { CHUNK_HEIGHT } else { CHUNK_SIZE };
    let b_dim = if tb == 1 { CHUNK_HEIGHT } else { CHUNK_SIZE };

    for plane in 0..n_dim {
        // The mask holds a compact merge key (the block state): two cells merge
        // only when their states match, never merely because textures match.
        let mut mask: Vec<Option<BlockState>> = vec![None; a_dim * b_dim];
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
                // Opaque full cubes are meshed word-parallel (M16); this path
                // handles transparent full cubes (glass) only.
                if def.shape != Shape::FullCube || !is_transparent(state) {
                    continue;
                }
                let neighbor = world.get_block(
                    base_x + x as i32 + nx,
                    y as i32 + ny,
                    base_z + z as i32 + nz,
                );
                if !face_visible(state, neighbor, [nx, ny, nz], Vec3::ZERO, Vec3::ONE) {
                    continue;
                }
                mask[a * b_dim + b] = Some(state);
            }
        }

        let mut visited = vec![false; a_dim * b_dim];
        for a in 0..a_dim {
            for b in 0..b_dim {
                let idx = a * b_dim + b;
                if visited[idx] {
                    continue;
                }
                let Some(state) = mask[idx] else {
                    continue;
                };

                let mut w = 1;
                while a + w < a_dim
                    && !visited[(a + w) * b_dim + b]
                    && mask[(a + w) * b_dim + b] == Some(state)
                {
                    w += 1;
                }
                let mut h = 1;
                'grow: while b + h < b_dim {
                    for da in 0..w {
                        if visited[(a + da) * b_dim + (b + h)]
                            || mask[(a + da) * b_dim + (b + h)] != Some(state)
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

                let desc = face_table.get(state, face);
                if let Some(rect) = desc.rect {
                    // Tiled UV (w x h tiles); the shader wraps it into `rect`.
                    let uvs = [0usize, 1, 2, 3].map(|i| {
                        let [u, v] = FACE_UVS[face][i];
                        [u * w as f32, v * h as f32]
                    });
                    target.add_face(corners, uvs, desc.color, rect);
                } else {
                    target.add_face(corners, [[0.0, 0.0]; 4], desc.color, [0.0, 0.0, 1.0, 1.0]);
                }
            }
        }
    }
}

/// Word-parallel meshing of opaque full-cube faces (M16): build `u32`
/// occupancy rows, derive exposed faces with integer AND/NOT, then greedily
/// merge by material. Transparent full cubes (glass) and slabs use the
/// shape-aware path.
#[allow(clippy::too_many_arguments)]
fn greedy_opaque_face(
    snapshot: &ChunkSnapshot,
    face_table: &FaceTable,
    base_x: i32,
    base_z: i32,
    face: usize,
    opaque: &mut MeshData,
) {
    let (na, ta, tb) = FACE_AXES[face];
    let n_dim = if na == 1 { CHUNK_HEIGHT } else { CHUNK_SIZE };
    let a_dim = if ta == 1 { CHUNK_HEIGHT } else { CHUNK_SIZE };
    let b_dim = if tb == 1 { CHUNK_HEIGHT } else { CHUNK_SIZE };
    let dir = FACE_NORMALS[face][na];

    // Occupancy rows: occ[plane+1][b] = u32, bit `a` = opaque full cube at the
    // cell (na=plane, ta=a, tb=b). A halo plane on each side makes boundary
    // exposure (a neighbour across the chunk edge) read as air.
    let mut occ: Vec<Vec<u32>> = vec![vec![0u32; b_dim]; n_dim + 2];
    for p in -1..=(n_dim as i32) {
        let plane_rows = &mut occ[(p + 1) as usize];
        for (b, slot) in plane_rows.iter_mut().enumerate() {
            let mut row = 0u32;
            for a in 0..a_dim {
                let mut c = [0i32; 3];
                c[na] = p;
                c[ta] = a as i32;
                c[tb] = b as i32;
                if occludes(snapshot.get(c[0], c[1], c[2])) {
                    row |= 1 << a;
                }
            }
            *slot = row;
        }
    }

    // Read the block state at a cell in this face's (na, ta, tb) frame.
    let cell = |plane: i32, a: i32, b: i32| -> BlockState {
        let mut c = [0i32; 3];
        c[na] = plane;
        c[ta] = a;
        c[tb] = b;
        snapshot.get(c[0], c[1], c[2])
    };

    for plane in 0..n_dim {
        let cur = plane + 1;
        let neighbor = (plane as i32 + dir + 1) as usize;

        // Exposed faces: occ[cur] AND NOT occ[neighbour].
        let exposed: Vec<u32> = occ[cur]
            .iter()
            .zip(&occ[neighbor])
            .map(|(&c, &n)| c & !n)
            .collect();

        let mut visited = vec![false; a_dim * b_dim];
        for b in 0..b_dim {
            for a in 0..a_dim {
                let idx = b * a_dim + a;
                if visited[idx] || exposed[b] & (1 << a) == 0 {
                    continue;
                }
                let state = cell(plane as i32, a as i32, b as i32);

                let mut w = 1;
                while a + w < a_dim
                    && !visited[b * a_dim + (a + w)]
                    && exposed[b] & (1 << (a + w)) != 0
                    && cell(plane as i32, (a + w) as i32, b as i32) == state
                {
                    w += 1;
                }
                let mut h = 1;
                'grow: while b + h < b_dim {
                    for da in 0..w {
                        let aa = a + da;
                        if visited[(b + h) * a_dim + aa]
                            || exposed[b + h] & (1 << aa) == 0
                            || cell(plane as i32, aa as i32, (b + h) as i32) != state
                        {
                            break 'grow;
                        }
                    }
                    h += 1;
                }
                for db in 0..h {
                    for da in 0..w {
                        visited[(b + db) * a_dim + (a + da)] = true;
                    }
                }

                let n_coord = plane as f32 + if dir > 0 { 1.0 } else { 0.0 };
                let mut mn = [0.0f32; 3];
                let mut mx = [0.0f32; 3];
                mn[na] = n_coord;
                mx[na] = n_coord;
                mn[ta] = a as f32;
                mx[ta] = (a + w) as f32;
                mn[tb] = b as f32;
                mx[tb] = (b + h) as f32;
                mn[0] += base_x as f32;
                mn[2] += base_z as f32;
                mx[0] += base_x as f32;
                mx[2] += base_z as f32;
                let corners = face_corners(Vec3::from_array(mn), Vec3::from_array(mx), face);

                let desc = face_table.get(state, face);
                if let Some(rect) = desc.rect {
                    let uvs = [0usize, 1, 2, 3].map(|i| {
                        let [u, v] = FACE_UVS[face][i];
                        [u * w as f32, v * h as f32]
                    });
                    opaque.add_face(corners, uvs, desc.color, rect);
                } else {
                    opaque.add_face(corners, [[0.0, 0.0]; 4], desc.color, [0.0, 0.0, 1.0, 1.0]);
                }
            }
        }
    }
}

/// Build the opaque and transparent meshes for one chunk (world-space positions).
pub fn build_chunk_meshes(
    world: &World,
    chunk: &Chunk,
    face_table: &FaceTable,
) -> (MeshData, MeshData) {
    let mut opaque = MeshData::new();
    let mut transparent = MeshData::new();

    let base_x = chunk.cx * CHUNK_SIZE as i32;
    let base_z = chunk.cz * CHUNK_SIZE as i32;
    let snapshot = ChunkSnapshot::gather(world, chunk);

    // Full cubes: opaque word-parallel (M16), then glass per-cell.
    for face in 0..6 {
        greedy_opaque_face(&snapshot, face_table, base_x, base_z, face, &mut opaque);
        greedy_face(
            world,
            chunk,
            face_table,
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
                    if !face_visible(state, neighbor, [nx, ny, nz], bmin, bmax) {
                        continue;
                    }
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

                    let desc = face_table.get(state, face);
                    if let Some(rect) = desc.rect {
                        let uvs = FACE_UVS[face].map(|[u, v]| {
                            let v = match def.shape {
                                Shape::SlabBottom if face >= 2 => 0.5 + 0.5 * v,
                                Shape::SlabTop if face >= 2 => 0.5 * v,
                                _ => v,
                            };
                            [u, v]
                        });
                        target.add_face(corners, uvs, desc.color, rect);
                    } else {
                        target.add_face(corners, [[0.0, 0.0]; 4], desc.color, [0.0, 0.0, 1.0, 1.0]);
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
    use sunset_core::block::state_by_id;
    use sunset_core::fixture;

    /// The packed vertex is 28 bytes: 12 (position) + 4 (uv) + 4 (color) + 8
    /// (rect), down from 52. Guards against accidental regrowth.
    #[test]
    fn vertex_is_compact() {
        assert_eq!(std::mem::size_of::<Vertex>(), 28);
    }

    /// A boundary edit (changing a block on a chunk edge) must produce, after
    /// re-meshing only the affected chunk and its four neighbours, exactly the
    /// same meshes as a full rebuild of every chunk.
    #[test]
    fn boundary_edit_matches_full_rebuild() {
        let face_table = FaceTable::build(&HashMap::new(), &HashMap::new());

        let mut world = fixture::building_yard();

        // Pre-edit meshes for every chunk (the "incremental" starting point).
        let mut incremental: HashMap<(i32, i32), (MeshData, MeshData)> = HashMap::new();
        for chunk in world.chunks() {
            incremental.insert(
                (chunk.cx, chunk.cz),
                build_chunk_meshes(&world, chunk, &face_table),
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
                    build_chunk_meshes(&world, chunk, &face_table),
                );
            }
        }

        // Full rebuild reference after the edit.
        let mut full: HashMap<(i32, i32), (MeshData, MeshData)> = HashMap::new();
        for chunk in world.chunks() {
            full.insert(
                (chunk.cx, chunk.cz),
                build_chunk_meshes(&world, chunk, &face_table),
            );
        }

        assert_eq!(incremental.len(), full.len(), "chunk set must match");
        for (key, (o, t)) in &full {
            let (io, it) = &incremental[key];
            assert_eq!(io, o, "opaque mesh mismatch for chunk {key:?}");
            assert_eq!(it, t, "transparent mesh mismatch for chunk {key:?}");
        }
    }

    /// (centroid, unit outward normal) per triangle.
    fn triangle_centroids(mesh: &MeshData) -> Vec<([f32; 3], [f32; 3])> {
        mesh.indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|tri| {
                let a = mesh.vertices[tri[0] as usize].position;
                let b = mesh.vertices[tri[1] as usize].position;
                let c = mesh.vertices[tri[2] as usize].position;
                let centroid = [
                    (a[0] + b[0] + c[0]) / 3.0,
                    (a[1] + b[1] + c[1]) / 3.0,
                    (a[2] + b[2] + c[2]) / 3.0,
                ];
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let n = [
                    e1[1] * e2[2] - e1[2] * e2[1],
                    e1[2] * e2[0] - e1[0] * e2[2],
                    e1[0] * e2[1] - e1[1] * e2[0],
                ];
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                (centroid, [n[0] / len, n[1] / len, n[2] / len])
            })
            .collect()
    }

    /// Snap a unit normal to its axis direction (`[±1, 0, 0]`, …).
    fn axis_of(n: [f32; 3]) -> Option<[i32; 3]> {
        for a in 0..3 {
            if n[a].abs() > 0.9 {
                let mut v = [0i32; 3];
                v[a] = if n[a] > 0.0 { 1 } else { -1 };
                return Some(v);
            }
        }
        None
    }

    /// Mesh one chunk of a small authored world (no textures/atlas).
    fn mesh_world(world: &World, cx: i32, cz: i32) -> (MeshData, MeshData) {
        let face_table = FaceTable::build(&HashMap::new(), &HashMap::new());
        let chunk = world.get_chunk(cx, cz).expect("chunk exists");
        build_chunk_meshes(world, chunk, &face_table)
    }

    #[test]
    fn cube_winds_outward() {
        let stone = state_by_id("minecraft:stone").expect("registered");
        let mut world = World::new();
        world.set_block(0, 0, 0, stone);
        let (opaque, _) = mesh_world(&world, 0, 0);

        assert_eq!(opaque.indices.len() / 3, 12, "single cube = 6 faces = 12 tris");

        let mut counts: HashMap<[i32; 3], usize> = HashMap::new();
        for &(_, n) in &triangle_centroids(&opaque) {
            *counts.entry(axis_of(n).expect("axis-aligned")).or_default() += 1;
        }
        for dir in [[0, 1, 0], [0, -1, 0], [1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1]] {
            assert_eq!(counts.get(&dir), Some(&2), "face {dir:?} must wind outward");
        }
    }

    #[test]
    fn bottom_slab_top_face_not_culled_by_cube_above() {
        let slab = state_by_id("minecraft:smooth_stone_slab[type=bottom]").expect("registered");
        let stone = state_by_id("minecraft:stone").expect("registered");
        let mut world = World::new();
        world.set_block(0, 0, 0, slab);
        world.set_block(0, 1, 0, stone);
        let (opaque, _) = mesh_world(&world, 0, 0);

        let has_slab_top = triangle_centroids(&opaque)
            .iter()
            .any(|&(c, n)| axis_of(n) == Some([0, 1, 0]) && (c[1] - 0.5).abs() < 0.01);
        assert!(
            has_slab_top,
            "bottom slab top face (y=0.5) must not be culled by the cube above"
        );
    }

    #[test]
    fn top_slab_bottom_face_not_culled_by_cube_below() {
        let slab = state_by_id("minecraft:smooth_stone_slab[type=top]").expect("registered");
        let stone = state_by_id("minecraft:stone").expect("registered");
        let mut world = World::new();
        world.set_block(0, 0, 0, stone);
        world.set_block(0, 1, 0, slab);
        let (opaque, _) = mesh_world(&world, 0, 0);

        let has_slab_bottom = triangle_centroids(&opaque)
            .iter()
            .any(|&(c, n)| axis_of(n) == Some([0, -1, 0]) && (c[1] - 1.5).abs() < 0.01);
        assert!(
            has_slab_bottom,
            "top slab bottom face (y=1.5) must not be culled by the cube below"
        );
    }

    #[test]
    fn stacked_cubes_cull_interface_face() {
        let stone = state_by_id("minecraft:stone").expect("registered");
        let mut world = World::new();
        world.set_block(0, 0, 0, stone);
        world.set_block(0, 1, 0, stone);
        let (opaque, _) = mesh_world(&world, 0, 0);

        // Greedy meshing merges the coplanar side faces of the two cubes, so:
        // 1 top (+Y) + 1 bottom (-Y) + 4 merged side faces = 6 faces = 12 tris.
        assert_eq!(opaque.indices.len() / 3, 12, "two stacked cubes expose 6 faces");

        // The interface at y=1 (bottom cube's top / top cube's bottom) is culled:
        // no +Y or -Y face sits at y=1.
        let interface = triangle_centroids(&opaque).iter().any(|&(c, n)| {
            (c[1] - 1.0).abs() < 0.01
                && (axis_of(n) == Some([0, 1, 0]) || axis_of(n) == Some([0, -1, 0]))
        });
        assert!(!interface, "interface face between stacked cubes must be culled");
    }

    #[test]
    fn glass_renders_all_six_faces_transparent() {
        let glass = state_by_id("minecraft:glass").expect("registered");
        let mut world = World::new();
        world.set_block(0, 0, 0, glass);
        let (opaque, transparent) = mesh_world(&world, 0, 0);

        assert!(opaque.indices.is_empty(), "glass is not opaque");
        assert_eq!(transparent.indices.len() / 3, 12, "glass exposes all 6 faces");
    }

    #[test]
    fn glass_bottom_face_culled_on_solid_ground() {
        let glass = state_by_id("minecraft:glass").expect("registered");
        let stone = state_by_id("minecraft:stone").expect("registered");
        let mut world = World::new();
        world.set_block(0, 0, 0, stone); // solid ground
        world.set_block(0, 1, 0, glass); // glass on top
        let (_, transparent) = mesh_world(&world, 0, 0);

        // The glass's bottom face (normal -Y at y=1) must not be emitted: it
        // would z-fight with the solid block's top face below.
        let glass_bottom = triangle_centroids(&transparent).iter().any(|&(c, n)| {
            axis_of(n) == Some([0, -1, 0]) && (c[1] - 1.0).abs() < 0.01
        });
        assert!(!glass_bottom, "glass bottom face z-fights with solid ground");
    }

    #[test]
    fn merged_rectangle_top_face() {
        let stone = state_by_id("minecraft:stone").expect("registered");
        let mut world = World::new();
        for wx in 0..2 {
            for wz in 0..2 {
                world.set_block(wx, 0, wz, stone);
            }
        }
        let (opaque, _) = mesh_world(&world, 0, 0);

        let top = triangle_centroids(&opaque)
            .iter()
            .filter(|&&(_, n)| axis_of(n) == Some([0, 1, 0]))
            .count();
        assert_eq!(top, 2, "2x2 top surface must merge into a single rectangle");
    }
}

