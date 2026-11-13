use crate::culling::{Aabb, Frustum};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct DrawCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct SectionDraw {
    pub aabb: Aabb,
    pub index_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawMode {
    Direct,
    MultiDrawIndirect,
    LoopedIndirect,
}

pub fn select_draw_mode(downlevel: wgpu::DownlevelFlags) -> DrawMode {
    if downlevel.contains(wgpu::DownlevelFlags::INDIRECT_EXECUTION) {
        DrawMode::MultiDrawIndirect
    } else {
        DrawMode::LoopedIndirect
    }
}

pub fn visible_sections(sections: &[SectionDraw], frustum: &Frustum) -> Vec<usize> {
    sections
        .iter()
        .enumerate()
        .filter(|(_, section)| !frustum.cull(&section.aabb))
        .map(|(index, _)| index)
        .collect()
}

pub fn build_indirect_commands(sections: &[SectionDraw], visible: &[usize]) -> Vec<DrawCommand> {
    visible
        .iter()
        .map(|&index| {
            let section = sections[index];
            DrawCommand {
                index_count: section.index_count,
                instance_count: 1,
                first_index: section.first_index,
                base_vertex: section.base_vertex,
                first_instance: index as u32,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::culling::Plane;

    #[test]
    fn selects_multi_draw_indirect_when_supported() {
        let flags = wgpu::DownlevelFlags::INDIRECT_EXECUTION;
        assert_eq!(select_draw_mode(flags), DrawMode::MultiDrawIndirect);
        assert_eq!(
            select_draw_mode(wgpu::DownlevelFlags::empty()),
            DrawMode::LoopedIndirect
        );
    }

    #[test]
    fn visible_sections_matches_brute_force_and_is_contiguous() {
        let frustum = box_frustum(8.0);
        let mut sections = Vec::new();
        for x in -4..4 {
            for z in -4..4 {
                sections.push(SectionDraw {
                    aabb: Aabb::new(
                        [x as f32 * 16.0, 0.0, z as f32 * 16.0],
                        [x as f32 * 16.0 + 16.0, 16.0, z as f32 * 16.0 + 16.0],
                    ),
                    index_count: 6,
                    first_index: (x * 64 + z) as u32,
                    base_vertex: x * 64 + z * 8,
                });
            }
        }

        let visible = visible_sections(&sections, &frustum);
        let brute: Vec<usize> = sections
            .iter()
            .enumerate()
            .filter(|(_, section)| !frustum.cull(&section.aabb))
            .map(|(index, _)| index)
            .collect();
        assert_eq!(visible, brute);

        let commands = build_indirect_commands(&sections, &visible);
        assert_eq!(commands.len(), visible.len());
        for (command, &section_index) in commands.iter().zip(&visible) {
            let section = sections[section_index];
            assert_eq!(command.index_count, section.index_count);
            assert_eq!(command.first_index, section.first_index);
            assert_eq!(command.base_vertex, section.base_vertex);
            assert_eq!(command.instance_count, 1);
            assert_eq!(command.first_instance, section_index as u32);
        }
    }

    #[test]
    fn direct_and_indirect_paths_share_the_same_section_set() {
        let frustum = box_frustum(8.0);
        let sections = vec![
            SectionDraw {
                aabb: Aabb::new([0.0, 0.0, 0.0], [16.0, 16.0, 16.0]),
                index_count: 6,
                first_index: 0,
                base_vertex: 0,
            },
            SectionDraw {
                aabb: Aabb::new([100.0, 0.0, 0.0], [116.0, 16.0, 16.0]),
                index_count: 6,
                first_index: 6,
                base_vertex: 4,
            },
        ];
        let visible = visible_sections(&sections, &frustum);
        assert_eq!(visible, vec![0]);
        let commands = build_indirect_commands(&sections, &visible);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].first_index, 0);
    }

    fn box_frustum(half: f32) -> Frustum {
        Frustum {
            planes: [
                Plane {
                    normal: [1.0, 0.0, 0.0],
                    distance: half,
                },
                Plane {
                    normal: [-1.0, 0.0, 0.0],
                    distance: half,
                },
                Plane {
                    normal: [0.0, 1.0, 0.0],
                    distance: half,
                },
                Plane {
                    normal: [0.0, -1.0, 0.0],
                    distance: half,
                },
                Plane {
                    normal: [0.0, 0.0, 1.0],
                    distance: half,
                },
                Plane {
                    normal: [0.0, 0.0, -1.0],
                    distance: half,
                },
            ],
        }
    }
}
