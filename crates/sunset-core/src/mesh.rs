use crate::block::AIR;
use crate::palette::PalettedSection;

pub const VERTEX_SIZE_BYTES: usize = 8;

const _: () = assert!(VERTEX_SIZE_BYTES <= 16);

const POS_X_SHIFT: u32 = 0;
const POS_Y_SHIFT: u32 = 5;
const POS_Z_SHIFT: u32 = 10;
const FACE_SHIFT: u32 = 15;
const AO_SHIFT: u32 = 18;
const LIGHT_SHIFT: u32 = 20;
const UV_INDEX_SHIFT: u32 = 24;
const TINT_INDEX_SHIFT: u32 = 40;

const POS_MASK: u64 = 0x1F;
const FACE_MASK: u64 = 0x7;
const AO_MASK: u64 = 0x3;
const LIGHT_MASK: u64 = 0xF;
const UV_INDEX_MASK: u64 = 0xFFFF;
const TINT_INDEX_MASK: u64 = 0xFF;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Face {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::Down,
        Face::Up,
        Face::North,
        Face::South,
        Face::West,
        Face::East,
    ];
}

#[derive(Clone, Copy)]
struct FaceAxes {
    normal: usize,
    u: usize,
    v: usize,
    positive: bool,
}

const FACE_AXES: [FaceAxes; 6] = [
    FaceAxes {
        normal: 1,
        u: 0,
        v: 2,
        positive: false,
    },
    FaceAxes {
        normal: 1,
        u: 0,
        v: 2,
        positive: true,
    },
    FaceAxes {
        normal: 2,
        u: 0,
        v: 1,
        positive: false,
    },
    FaceAxes {
        normal: 2,
        u: 0,
        v: 1,
        positive: true,
    },
    FaceAxes {
        normal: 0,
        u: 1,
        v: 2,
        positive: false,
    },
    FaceAxes {
        normal: 0,
        u: 1,
        v: 2,
        positive: true,
    },
];

#[derive(Clone, Copy)]
struct Quad {
    n: u8,
    u0: u8,
    v0: u8,
    u_len: u8,
    v_len: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Vertex(u64);

impl Vertex {
    pub const fn new(
        pos: [u8; 3],
        face: Face,
        ao: u8,
        light: u8,
        uv_index: u16,
        tint_index: u8,
    ) -> Self {
        let packed = ((pos[0] as u64 & POS_MASK) << POS_X_SHIFT)
            | ((pos[1] as u64 & POS_MASK) << POS_Y_SHIFT)
            | ((pos[2] as u64 & POS_MASK) << POS_Z_SHIFT)
            | ((face as u64 & FACE_MASK) << FACE_SHIFT)
            | ((ao as u64 & AO_MASK) << AO_SHIFT)
            | ((light as u64 & LIGHT_MASK) << LIGHT_SHIFT)
            | ((uv_index as u64 & UV_INDEX_MASK) << UV_INDEX_SHIFT)
            | ((tint_index as u64 & TINT_INDEX_MASK) << TINT_INDEX_SHIFT);
        Self(packed)
    }

    pub const fn pos(self) -> [u8; 3] {
        [
            ((self.0 >> POS_X_SHIFT) & POS_MASK) as u8,
            ((self.0 >> POS_Y_SHIFT) & POS_MASK) as u8,
            ((self.0 >> POS_Z_SHIFT) & POS_MASK) as u8,
        ]
    }

    pub const fn face(self) -> Face {
        match (self.0 >> FACE_SHIFT) & FACE_MASK {
            0 => Face::Down,
            1 => Face::Up,
            2 => Face::North,
            3 => Face::South,
            4 => Face::West,
            _ => Face::East,
        }
    }

    pub const fn ao(self) -> u8 {
        ((self.0 >> AO_SHIFT) & AO_MASK) as u8
    }

    pub const fn light(self) -> u8 {
        ((self.0 >> LIGHT_SHIFT) & LIGHT_MASK) as u8
    }

    pub const fn uv_index(self) -> u16 {
        ((self.0 >> UV_INDEX_SHIFT) & UV_INDEX_MASK) as u16
    }

    pub const fn tint_index(self) -> u8 {
        ((self.0 >> TINT_INDEX_SHIFT) & TINT_INDEX_MASK) as u8
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn to_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

#[derive(Default)]
pub struct SectionNeighbors<'a> {
    pub down: Option<&'a PalettedSection>,
    pub up: Option<&'a PalettedSection>,
    pub north: Option<&'a PalettedSection>,
    pub south: Option<&'a PalettedSection>,
    pub west: Option<&'a PalettedSection>,
    pub east: Option<&'a PalettedSection>,
}

const FACE_CORNERS: [[[u8; 3]; 4]; 6] = [
    [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]],
    [[0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]],
    [[0, 1, 0], [1, 1, 0], [1, 0, 0], [0, 0, 0]],
    [[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]],
    [[0, 0, 1], [0, 1, 1], [0, 1, 0], [0, 0, 0]],
    [[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]],
];

pub fn mesh_section(section: &PalettedSection, neighbors: &SectionNeighbors) -> Mesh {
    let mut mesh = Mesh::default();
    for y in 0..16u8 {
        for z in 0..16u8 {
            for x in 0..16u8 {
                let state = section.get(index(y, z, x));
                if state == AIR {
                    continue;
                }
                let uv_index = state;
                for &face in &Face::ALL {
                    if face_neighbor(section, neighbors, face, x, y, z) != AIR {
                        continue;
                    }
                    emit_face(&mut mesh, x, y, z, face, uv_index);
                }
            }
        }
    }
    mesh
}

fn face_neighbor(
    section: &PalettedSection,
    neighbors: &SectionNeighbors,
    face: Face,
    x: u8,
    y: u8,
    z: u8,
) -> u16 {
    match face {
        Face::Down => {
            if y == 0 {
                neighbors.down.map_or(AIR, |s| s.get(index(15, z, x)))
            } else {
                section.get(index(y - 1, z, x))
            }
        }
        Face::Up => {
            if y == 15 {
                neighbors.up.map_or(AIR, |s| s.get(index(0, z, x)))
            } else {
                section.get(index(y + 1, z, x))
            }
        }
        Face::North => {
            if z == 0 {
                neighbors.north.map_or(AIR, |s| s.get(index(y, 15, x)))
            } else {
                section.get(index(y, z - 1, x))
            }
        }
        Face::South => {
            if z == 15 {
                neighbors.south.map_or(AIR, |s| s.get(index(y, 0, x)))
            } else {
                section.get(index(y, z + 1, x))
            }
        }
        Face::West => {
            if x == 0 {
                neighbors.west.map_or(AIR, |s| s.get(index(y, z, 15)))
            } else {
                section.get(index(y, z, x - 1))
            }
        }
        Face::East => {
            if x == 15 {
                neighbors.east.map_or(AIR, |s| s.get(index(y, z, 0)))
            } else {
                section.get(index(y, z, x + 1))
            }
        }
    }
}

pub fn mesh_section_greedy(section: &PalettedSection, neighbors: &SectionNeighbors) -> Mesh {
    let mut mesh = Mesh::default();
    for &face in &Face::ALL {
        greedy_axis(&mut mesh, section, neighbors, face);
    }
    mesh
}

fn greedy_axis(
    mesh: &mut Mesh,
    section: &PalettedSection,
    neighbors: &SectionNeighbors,
    face: Face,
) {
    let axes = FACE_AXES[face as usize];
    for n in 0..16u8 {
        let mut mask: [[Option<u16>; 16]; 16] = [[None; 16]; 16];
        for u in 0..16u8 {
            for v in 0..16u8 {
                let pos = pos_from(axes, n, u, v);
                let state = section.get(index(pos[1], pos[2], pos[0]));
                if state == AIR {
                    continue;
                }
                if face_neighbor(section, neighbors, face, pos[0], pos[1], pos[2]) == AIR {
                    mask[u as usize][v as usize] = Some(state);
                }
            }
        }
        let mut consumed = [[false; 16]; 16];
        for u in 0..16usize {
            for v in 0..16usize {
                let Some(state) = mask[u][v] else {
                    continue;
                };
                if consumed[u][v] {
                    continue;
                }
                let mut v_len = 1;
                while v + v_len < 16 && mask[u][v + v_len] == Some(state) && !consumed[u][v + v_len]
                {
                    v_len += 1;
                }
                let mut u_len = 1;
                'extend: while u + u_len < 16 {
                    for dv in 0..v_len {
                        if mask[u + u_len][v + dv] != Some(state) || consumed[u + u_len][v + dv] {
                            break 'extend;
                        }
                    }
                    u_len += 1;
                }
                for du in 0..u_len {
                    for dv in 0..v_len {
                        consumed[u + du][v + dv] = true;
                    }
                }
                emit_quad(
                    mesh,
                    face,
                    Quad {
                        n,
                        u0: u as u8,
                        v0: v as u8,
                        u_len: u_len as u8,
                        v_len: v_len as u8,
                    },
                    state,
                );
            }
        }
    }
}

fn pos_from(axes: FaceAxes, n: u8, u: u8, v: u8) -> [u8; 3] {
    let mut pos = [0u8; 3];
    pos[axes.normal] = n;
    pos[axes.u] = u;
    pos[axes.v] = v;
    pos
}

fn emit_quad(mesh: &mut Mesh, face: Face, quad: Quad, uv_index: u16) {
    let axes = FACE_AXES[face as usize];
    let corners = FACE_CORNERS[face as usize];
    let base = mesh.vertices.len() as u32;
    for offset in corners {
        let mut corner = [0u8; 3];
        corner[axes.normal] = if axes.positive { quad.n + 1 } else { quad.n };
        corner[axes.u] = if offset[axes.u] == 0 {
            quad.u0
        } else {
            quad.u0 + quad.u_len
        };
        corner[axes.v] = if offset[axes.v] == 0 {
            quad.v0
        } else {
            quad.v0 + quad.v_len
        };
        mesh.vertices
            .push(Vertex::new(corner, face, 0, 15, uv_index, 0));
    }
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn emit_face(mesh: &mut Mesh, x: u8, y: u8, z: u8, face: Face, uv_index: u16) {
    let axes = FACE_AXES[face as usize];
    let pos = [x, y, z];
    emit_quad(
        mesh,
        face,
        Quad {
            n: pos[axes.normal],
            u0: pos[axes.u],
            v0: pos[axes.v],
            u_len: 1,
            v_len: 1,
        },
        uv_index,
    );
}

fn index(y: u8, z: u8, x: u8) -> usize {
    ((y as usize) << 8) | ((z as usize) << 4) | (x as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{STONE, WATER};
    use crate::palette::SECTION_SIZE;
    use crate::terrain::{TerrainGenerator, TerrainParams};
    use std::collections::BTreeSet;

    #[test]
    fn vertex_size_is_within_budget() {
        assert_eq!(std::mem::size_of::<Vertex>(), VERTEX_SIZE_BYTES);
    }

    #[test]
    fn vertex_quantization_round_trips() {
        let zero = Vertex::new([0, 0, 0], Face::Down, 0, 0, 0, 0);
        assert_eq!(zero.bits(), 0);

        let vertex = Vertex::new([0, 16, 8], Face::East, 3, 15, 0xFFFF, 0xFF);
        assert_eq!(vertex.pos(), [0, 16, 8]);
        assert_eq!(vertex.face(), Face::East);
        assert_eq!(vertex.ao(), 3);
        assert_eq!(vertex.light(), 15);
        assert_eq!(vertex.uv_index(), 0xFFFF);
        assert_eq!(vertex.tint_index(), 0xFF);
        assert_eq!(vertex.bits() >> 48, 0);

        let bytes = vertex.to_bytes();
        assert_eq!(bytes.len(), VERTEX_SIZE_BYTES);
        assert_eq!(u64::from_le_bytes(bytes), vertex.bits());
    }

    #[test]
    fn single_block_emits_six_outward_faces() {
        let mut section = PalettedSection::new(AIR);
        section.set(index(0, 0, 0), STONE);
        let mesh = mesh_section(&section, &SectionNeighbors::default());
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        assert_eq!(mesh.triangle_count(), 12);

        let mut normals: Vec<[i32; 3]> = Vec::new();
        for tri in mesh.indices.chunks(3) {
            let a = mesh.vertices[tri[0] as usize].pos();
            let b = mesh.vertices[tri[1] as usize].pos();
            let c = mesh.vertices[tri[2] as usize].pos();
            normals.push(triangle_normal(a, b, c));
        }
        normals.sort_unstable();

        let mut expected = Vec::new();
        for normal in [
            [-1, 0, 0],
            [1, 0, 0],
            [0, -1, 0],
            [0, 1, 0],
            [0, 0, -1],
            [0, 0, 1],
        ] {
            expected.push(normal);
            expected.push(normal);
        }
        expected.sort_unstable();
        assert_eq!(normals, expected);
    }

    #[test]
    fn slab_of_blocks_culls_shared_faces() {
        let mut section = PalettedSection::new(AIR);
        section.set(index(0, 0, 0), STONE);
        section.set(index(0, 1, 0), STONE);
        let mesh = mesh_section(&section, &SectionNeighbors::default());
        assert_eq!(mesh.vertices.len(), 40);
        assert_eq!(mesh.indices.len(), 60);
        assert_eq!(mesh.triangle_count(), 20);
    }

    #[test]
    fn water_is_rendered_as_a_full_cube() {
        let mut section = PalettedSection::new(AIR);
        section.set(index(0, 0, 0), WATER);
        let mesh = mesh_section(&section, &SectionNeighbors::default());
        assert_eq!(mesh.vertices.len(), 24);
    }

    #[test]
    fn section_border_culling_across_all_directions() {
        for face in Face::ALL {
            let (cx, cy, cz, nx, ny, nz) = match face {
                Face::Down => (0, 0, 0, 0, 15, 0),
                Face::Up => (0, 15, 0, 0, 0, 0),
                Face::North => (0, 0, 0, 0, 0, 15),
                Face::South => (0, 0, 15, 0, 0, 0),
                Face::West => (0, 0, 0, 15, 0, 0),
                Face::East => (15, 0, 0, 0, 0, 0),
            };
            let mut center = PalettedSection::new(AIR);
            center.set(index(cy, cz, cx), STONE);
            let mut neighbor_section = PalettedSection::new(AIR);
            neighbor_section.set(index(ny, nz, nx), STONE);
            let mut neighbors = SectionNeighbors::default();
            set_neighbor(&mut neighbors, face, &neighbor_section);

            let mesh = mesh_section(&center, &neighbors);
            assert_eq!(mesh.vertices.len(), 20, "face {face:?}");
            assert_eq!(mesh.indices.len(), 30, "face {face:?}");
            assert!(
                !mesh.vertices.iter().any(|v| v.face() == face),
                "face {face:?} should be culled"
            );
        }
    }

    #[test]
    fn missing_neighbor_is_treated_as_air() {
        let mut center = PalettedSection::new(AIR);
        center.set(index(0, 0, 15), STONE);
        let mesh = mesh_section(&center, &SectionNeighbors::default());
        assert_eq!(mesh.vertices.len(), 24);
    }

    #[test]
    fn greedy_matches_reference_and_oracle_over_random_sections() {
        let mut state = 0x9e3779b97f4a7c15u64;
        for _ in 0..48 {
            let section = random_section(&mut state);
            let down = random_section(&mut state);
            let up = random_section(&mut state);
            let north = random_section(&mut state);
            let south = random_section(&mut state);
            let west = random_section(&mut state);
            let east = random_section(&mut state);
            let neighbors = SectionNeighbors {
                down: Some(&down),
                up: Some(&up),
                north: Some(&north),
                south: Some(&south),
                west: Some(&west),
                east: Some(&east),
            };

            let reference = mesh_section(&section, &neighbors);
            let greedy = mesh_section_greedy(&section, &neighbors);
            let oracle = oracle_surface(&section, &neighbors);

            assert_eq!(mesh_to_surface(&reference), oracle);
            assert_eq!(mesh_to_surface(&greedy), oracle);
            assert!(greedy.triangle_count() <= reference.triangle_count());
        }
    }

    #[test]
    fn greedy_reduces_triangle_count_on_terrain() {
        let generator = TerrainGenerator::new(12345, TerrainParams::default());
        let section = generator.generate_section(0, 4, 0);
        let down = generator.generate_section(0, 3, 0);
        let up = generator.generate_section(0, 5, 0);
        let north = generator.generate_section(0, 4, -1);
        let south = generator.generate_section(0, 4, 1);
        let west = generator.generate_section(-1, 4, 0);
        let east = generator.generate_section(1, 4, 0);
        let neighbors = SectionNeighbors {
            down: Some(&down),
            up: Some(&up),
            north: Some(&north),
            south: Some(&south),
            west: Some(&west),
            east: Some(&east),
        };

        let reference = mesh_section(&section, &neighbors);
        let greedy = mesh_section_greedy(&section, &neighbors);

        assert_eq!(mesh_to_surface(&reference), mesh_to_surface(&greedy));
        assert!(greedy.triangle_count() < reference.triangle_count());
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    struct SurfaceFace {
        x: u8,
        y: u8,
        z: u8,
        face: u8,
        state: u16,
    }

    fn oracle_surface(
        section: &PalettedSection,
        neighbors: &SectionNeighbors,
    ) -> BTreeSet<SurfaceFace> {
        let mut set = BTreeSet::new();
        for y in 0..16u8 {
            for z in 0..16u8 {
                for x in 0..16u8 {
                    let state = section.get(index(y, z, x));
                    if state == AIR {
                        continue;
                    }
                    for &face in &Face::ALL {
                        if face_neighbor(section, neighbors, face, x, y, z) == AIR {
                            set.insert(SurfaceFace {
                                x,
                                y,
                                z,
                                face: face as u8,
                                state,
                            });
                        }
                    }
                }
            }
        }
        set
    }

    fn mesh_to_surface(mesh: &Mesh) -> BTreeSet<SurfaceFace> {
        let mut set = BTreeSet::new();
        for chunk in mesh.indices.as_chunks::<6>().0 {
            let base = chunk[0] as usize;
            let face = mesh.vertices[base].face();
            let state = mesh.vertices[base].uv_index();
            let axes = FACE_AXES[face as usize];
            let n = mesh.vertices[base].pos()[axes.normal];
            let mut u_min = 16u8;
            let mut u_max = 0u8;
            let mut v_min = 16u8;
            let mut v_max = 0u8;
            for offset in 0..4 {
                let pos = mesh.vertices[base + offset].pos();
                u_min = u_min.min(pos[axes.u]);
                u_max = u_max.max(pos[axes.u]);
                v_min = v_min.min(pos[axes.v]);
                v_max = v_max.max(pos[axes.v]);
            }
            let block_normal = if axes.positive { n - 1 } else { n };
            for u in u_min..u_max {
                for v in v_min..v_max {
                    let mut pos = [0u8; 3];
                    pos[axes.normal] = block_normal;
                    pos[axes.u] = u;
                    pos[axes.v] = v;
                    set.insert(SurfaceFace {
                        x: pos[0],
                        y: pos[1],
                        z: pos[2],
                        face: face as u8,
                        state,
                    });
                }
            }
        }
        set
    }

    fn random_section(state: &mut u64) -> PalettedSection {
        let mut section = PalettedSection::new(AIR);
        for cell in 0..SECTION_SIZE {
            let r = next_u64(state);
            if r.is_multiple_of(3) {
                continue;
            }
            let block = 1 + (r >> 3) % 8;
            section.set(cell, block as u16);
        }
        section
    }

    fn next_u64(state: &mut u64) -> u64 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        x
    }

    fn set_neighbor<'a>(
        neighbors: &mut SectionNeighbors<'a>,
        face: Face,
        section: &'a PalettedSection,
    ) {
        match face {
            Face::Down => neighbors.down = Some(section),
            Face::Up => neighbors.up = Some(section),
            Face::North => neighbors.north = Some(section),
            Face::South => neighbors.south = Some(section),
            Face::West => neighbors.west = Some(section),
            Face::East => neighbors.east = Some(section),
        }
    }

    fn triangle_normal(a: [u8; 3], b: [u8; 3], c: [u8; 3]) -> [i32; 3] {
        let ax = a[0] as i32;
        let ay = a[1] as i32;
        let az = a[2] as i32;
        let ux = b[0] as i32 - ax;
        let uy = b[1] as i32 - ay;
        let uz = b[2] as i32 - az;
        let vx = c[0] as i32 - ax;
        let vy = c[1] as i32 - ay;
        let vz = c[2] as i32 - az;
        [uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx]
    }
}
