#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub const fn section_x(self) -> i32 {
        self.x >> 4
    }

    pub const fn section_y(self) -> i32 {
        self.y >> 4
    }

    pub const fn section_z(self) -> i32 {
        self.z >> 4
    }

    pub const fn local_x(self) -> u32 {
        (self.x & 15) as u32
    }

    pub const fn local_y(self) -> u32 {
        (self.y & 15) as u32
    }

    pub const fn local_z(self) -> u32 {
        (self.z & 15) as u32
    }

    pub const fn section(self) -> SectionPos {
        SectionPos {
            x: self.section_x(),
            y: self.section_y(),
            z: self.section_z(),
        }
    }

    pub const fn chunk(self) -> ChunkPos {
        ChunkPos {
            x: self.x >> 4,
            z: self.z >> 4,
        }
    }

    pub const fn section_index(self) -> usize {
        ((self.local_y() << 8) | (self.local_z() << 4) | self.local_x()) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arithmetic::{floor_div, floor_mod};

    #[test]
    fn section_coordinates_use_floor_division() {
        let pos = BlockPos::new(-1, -16, -17);
        assert_eq!(pos.section_x(), -1);
        assert_eq!(pos.section_y(), -1);
        assert_eq!(pos.section_z(), -2);
        assert_eq!(pos.section_x(), floor_div(-1, 16));
        assert_eq!(pos.section_y(), floor_div(-16, 16));
        assert_eq!(pos.section_z(), floor_div(-17, 16));
    }

    #[test]
    fn local_coordinates_use_floor_modulo() {
        let pos = BlockPos::new(-1, -16, -17);
        assert_eq!(pos.local_x(), 15);
        assert_eq!(pos.local_y(), 0);
        assert_eq!(pos.local_z(), 15);
        assert_eq!(pos.local_x(), floor_mod(-1, 16) as u32);
        assert_eq!(pos.local_y(), floor_mod(-16, 16) as u32);
        assert_eq!(pos.local_z(), floor_mod(-17, 16) as u32);
    }

    #[test]
    fn extreme_coordinates_do_not_overflow() {
        let min = BlockPos::new(i32::MIN, i32::MIN, i32::MIN);
        assert_eq!(min.section_x(), i32::MIN >> 4);
        assert_eq!(min.section_y(), i32::MIN >> 4);
        assert_eq!(min.section_z(), i32::MIN >> 4);
        assert_eq!(min.local_x(), 0);
        let max = BlockPos::new(i32::MAX, i32::MAX, i32::MAX);
        assert_eq!(max.section_x(), i32::MAX >> 4);
        assert_eq!(max.local_x(), 15);
    }

    #[test]
    fn section_index_matches_yzx_order() {
        assert_eq!(BlockPos::new(0, 0, 0).section_index(), 0);
        assert_eq!(BlockPos::new(15, 0, 0).section_index(), 15);
        assert_eq!(BlockPos::new(0, 0, 1).section_index(), 16);
        assert_eq!(BlockPos::new(0, 1, 0).section_index(), 256);
        assert_eq!(BlockPos::new(15, 15, 15).section_index(), 4095);
    }

    #[test]
    fn property_block_to_section_round_trip() {
        let mut state = 0xa0761d6478bd642fu64;
        for _ in 0..10_000 {
            let x = next_u32(&mut state) as i32;
            let y = next_u32(&mut state) as i32;
            let z = next_u32(&mut state) as i32;
            let pos = BlockPos::new(x, y, z);
            let section = pos.section();
            let reconstructed_x = section
                .x
                .wrapping_mul(16)
                .wrapping_add(pos.local_x() as i32);
            assert_eq!(reconstructed_x, x);
            assert_eq!(section.x, floor_div(x, 16));
            assert_eq!(pos.local_x(), floor_mod(x, 16) as u32);
        }
    }

    fn next_u32(state: &mut u64) -> u32 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        x as u32
    }
}
