use crate::culling::Aabb;
use sunset_core::{
    mesh_section_greedy, PalettedSection, SectionNeighbors, TerrainGenerator, TerrainParams, Vertex,
};

pub struct SceneSection {
    pub aabb: Aabb,
    pub origin: [f32; 3],
    pub index_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
}

pub struct TerrainScene {
    pub vertices: Vec<u8>,
    pub indices: Vec<u32>,
    pub sections: Vec<SceneSection>,
}

pub fn build_terrain_scene(seed: u64, coords: &[(i32, i32, i32)]) -> TerrainScene {
    let generator = TerrainGenerator::new(seed, TerrainParams::default());
    let sections: Vec<PalettedSection> = coords
        .iter()
        .map(|&(sx, sy, sz)| generator.generate_section(sx, sy, sz))
        .collect();

    let mut scene = TerrainScene {
        vertices: Vec::new(),
        indices: Vec::new(),
        sections: Vec::new(),
    };
    for (index, &(sx, sy, sz)) in coords.iter().enumerate() {
        let neighbors = SectionNeighbors {
            down: find(&sections, coords, (sx, sy - 1, sz)),
            up: find(&sections, coords, (sx, sy + 1, sz)),
            north: find(&sections, coords, (sx, sy, sz - 1)),
            south: find(&sections, coords, (sx, sy, sz + 1)),
            west: find(&sections, coords, (sx - 1, sy, sz)),
            east: find(&sections, coords, (sx + 1, sy, sz)),
        };
        let mesh = mesh_section_greedy(&sections[index], &neighbors);
        let base_vertex = (scene.vertices.len() / std::mem::size_of::<Vertex>()) as i32;
        let first_index = scene.indices.len() as u32;
        for vertex in &mesh.vertices {
            scene.vertices.extend_from_slice(&vertex.to_bytes());
        }
        scene.indices.extend_from_slice(&mesh.indices);
        scene.sections.push(SceneSection {
            aabb: Aabb::new(
                [sx as f32 * 16.0, sy as f32 * 16.0, sz as f32 * 16.0],
                [
                    sx as f32 * 16.0 + 16.0,
                    sy as f32 * 16.0 + 16.0,
                    sz as f32 * 16.0 + 16.0,
                ],
            ),
            origin: [sx as f32 * 16.0, sy as f32 * 16.0, sz as f32 * 16.0],
            index_count: mesh.indices.len() as u32,
            first_index,
            base_vertex,
        });
    }
    scene
}

fn find<'a>(
    sections: &'a [PalettedSection],
    coords: &[(i32, i32, i32)],
    target: (i32, i32, i32),
) -> Option<&'a PalettedSection> {
    coords
        .iter()
        .position(|&coord| coord == target)
        .map(|index| &sections[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_section_has_expected_geometry_and_metadata() {
        let scene = build_terrain_scene(12345, &[(0, 4, 0)]);
        assert_eq!(scene.sections.len(), 1);
        assert!(scene.vertices.len() >= 8);
        assert!(scene.indices.len() >= 6);
        assert_eq!(scene.vertices.len() % std::mem::size_of::<Vertex>(), 0);

        let section = &scene.sections[0];
        assert_eq!(section.aabb.min, [0.0, 64.0, 0.0]);
        assert_eq!(section.aabb.max, [16.0, 80.0, 16.0]);
        assert_eq!(section.origin, [0.0, 64.0, 0.0]);
        assert_eq!(section.base_vertex, 0);
        assert_eq!(section.first_index, 0);
        assert_eq!(section.index_count as usize, scene.indices.len());
    }

    #[test]
    fn build_is_seed_deterministic() {
        let a = build_terrain_scene(7, &[(0, 4, 0), (1, 4, 0)]);
        let b = build_terrain_scene(7, &[(0, 4, 0), (1, 4, 0)]);
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.indices, b.indices);
    }

    #[test]
    fn adjacent_sections_accumulate_offsets() {
        let scene = build_terrain_scene(12345, &[(0, 4, 0), (1, 4, 0)]);
        assert_eq!(scene.sections.len(), 2);
        let first = &scene.sections[0];
        let second = &scene.sections[1];
        let first_vertex_count = (first.index_count / 6 * 4) as i32;
        assert_eq!(second.base_vertex, first_vertex_count);
        assert_eq!(second.first_index, first.index_count);
    }
}
