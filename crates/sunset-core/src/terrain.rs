use crate::block;
use crate::palette::PalettedSection;

#[derive(Clone, Copy)]
pub struct TerrainParams {
    pub base_height: i32,
    pub height_amplitude: f32,
    pub sea_level: i32,
    pub cave_threshold: f32,
    pub foliage_density: f32,
    pub snow_line: i32,
}

impl Default for TerrainParams {
    fn default() -> Self {
        Self {
            base_height: 64,
            height_amplitude: 20.0,
            sea_level: 62,
            cave_threshold: 0.55,
            foliage_density: 0.02,
            snow_line: 96,
        }
    }
}

pub struct TerrainGenerator {
    seed: u64,
    params: TerrainParams,
}

impl TerrainGenerator {
    pub fn new(seed: u64, params: TerrainParams) -> Self {
        Self { seed, params }
    }

    pub fn generate_section(&self, sx: i32, sy: i32, sz: i32) -> PalettedSection {
        let mut section = PalettedSection::new(block::AIR);
        let origin_x = sx.wrapping_mul(16);
        let origin_y = sy.wrapping_mul(16);
        let origin_z = sz.wrapping_mul(16);
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let world_x = origin_x.wrapping_add(x);
                    let world_y = origin_y.wrapping_add(y);
                    let world_z = origin_z.wrapping_add(z);
                    let state = self.block_at(world_x, world_y, world_z);
                    if state != block::AIR {
                        section.set(((y << 8) | (z << 4) | x) as usize, state);
                    }
                }
            }
        }
        section
    }

    fn block_at(&self, x: i32, y: i32, z: i32) -> u16 {
        let surface = self.height(x, z);
        if y > surface {
            if y <= self.params.sea_level {
                return block::WATER;
            }
            return self.tree_block_at(x, y, z);
        }
        if self.cave(x, y, z) && y < surface - 1 {
            return block::AIR;
        }
        if y == surface {
            if surface <= self.params.sea_level + 1 {
                return block::SAND;
            }
            if surface > self.params.snow_line {
                return block::SNOW;
            }
            return block::GRASS;
        }
        if y >= surface - 3 {
            return block::DIRT;
        }
        block::STONE
    }

    fn height(&self, x: i32, z: i32) -> i32 {
        let mut amplitude = 1.0f32;
        let mut frequency = 0.005f32;
        let mut total = 0.0f32;
        let mut total_amplitude = 0.0f32;
        for _ in 0..3 {
            let noise = value_noise2d(
                self.seed ^ 0x1111,
                x as f32 * frequency,
                z as f32 * frequency,
            );
            total += noise * amplitude;
            total_amplitude += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }
        let normalized = total / total_amplitude;
        let offset = (normalized - 0.5) * 2.0 * self.params.height_amplitude;
        (self.params.base_height as f32 + offset).round() as i32
    }

    fn cave(&self, x: i32, y: i32, z: i32) -> bool {
        let noise = value_noise3d(
            self.seed ^ 0x2222,
            x as f32 * 0.03,
            y as f32 * 0.03,
            z as f32 * 0.03,
        );
        noise < self.params.cave_threshold
    }

    fn tree_block_at(&self, x: i32, y: i32, z: i32) -> u16 {
        for dz in -2..=2 {
            for dx in -2..=2 {
                let root_x = x + dx;
                let root_z = z + dz;
                if !self.tree_at(root_x, root_z) {
                    continue;
                }
                let surface = self.height(root_x, root_z);
                if surface <= self.params.sea_level + 1 {
                    continue;
                }
                let trunk_height = self.trunk_height(root_x, root_z);
                if dx == 0 && dz == 0 && y > surface && y <= surface + trunk_height {
                    return block::LOG;
                }
                let canopy_top = surface + trunk_height;
                if y >= canopy_top - 1 && y <= canopy_top + 1 {
                    return block::LEAVES;
                }
            }
        }
        block::AIR
    }

    fn tree_at(&self, x: i32, z: i32) -> bool {
        let value = hash2(self.seed ^ 0x3333, x, z) as f32 / u32::MAX as f32;
        value < self.params.foliage_density
    }

    fn trunk_height(&self, x: i32, z: i32) -> i32 {
        3 + (hash2(self.seed ^ 0x4444, x, z) % 3) as i32
    }
}

fn value_noise2d(seed: u64, x: f32, z: f32) -> f32 {
    let xi = x.floor() as i32;
    let zi = z.floor() as i32;
    let xf = x - xi as f32;
    let zf = z - zi as f32;
    let u = smoothstep(xf);
    let v = smoothstep(zf);
    let v00 = hash2(seed, xi, zi) as f32 / u32::MAX as f32;
    let v10 = hash2(seed, xi + 1, zi) as f32 / u32::MAX as f32;
    let v01 = hash2(seed, xi, zi + 1) as f32 / u32::MAX as f32;
    let v11 = hash2(seed, xi + 1, zi + 1) as f32 / u32::MAX as f32;
    lerp(lerp(v00, v10, u), lerp(v01, v11, u), v)
}

fn value_noise3d(seed: u64, x: f32, y: f32, z: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    let xf = x - xi as f32;
    let yf = y - yi as f32;
    let zf = z - zi as f32;
    let u = smoothstep(xf);
    let v = smoothstep(yf);
    let w = smoothstep(zf);
    let c000 = hash3(seed, xi, yi, zi) as f32 / u32::MAX as f32;
    let c100 = hash3(seed, xi + 1, yi, zi) as f32 / u32::MAX as f32;
    let c010 = hash3(seed, xi, yi + 1, zi) as f32 / u32::MAX as f32;
    let c110 = hash3(seed, xi + 1, yi + 1, zi) as f32 / u32::MAX as f32;
    let c001 = hash3(seed, xi, yi, zi + 1) as f32 / u32::MAX as f32;
    let c101 = hash3(seed, xi + 1, yi, zi + 1) as f32 / u32::MAX as f32;
    let c011 = hash3(seed, xi, yi + 1, zi + 1) as f32 / u32::MAX as f32;
    let c111 = hash3(seed, xi + 1, yi + 1, zi + 1) as f32 / u32::MAX as f32;
    let x00 = lerp(c000, c100, u);
    let x10 = lerp(c010, c110, u);
    let x01 = lerp(c001, c101, u);
    let x11 = lerp(c011, c111, u);
    let y0 = lerp(x00, x10, v);
    let y1 = lerp(x01, x11, v);
    lerp(y0, y1, w)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn hash2(seed: u64, x: i32, z: i32) -> u32 {
    hash3(seed, x, 0, z)
}

fn hash3(seed: u64, x: i32, y: i32, z: i32) -> u32 {
    let mut h = seed;
    h = h.wrapping_add((x as u64).wrapping_mul(0x9E3779B97F4A7C15));
    h = h.wrapping_add((y as u64).wrapping_mul(0xC6A4A7935BD1E995));
    h = h.wrapping_add((z as u64).wrapping_mul(0x27D4EB2F165667C5));
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D049BB133111EB);
    h ^= h >> 31;
    h as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::SECTION_SIZE;

    #[test]
    fn generation_is_seed_deterministic() {
        let params = TerrainParams::default();
        let a = TerrainGenerator::new(42, params);
        let b = TerrainGenerator::new(42, params);
        let c = TerrainGenerator::new(43, params);
        assert_eq!(region_hash(&a), region_hash(&b));
        assert_ne!(region_hash(&a), region_hash(&c));
    }

    #[test]
    fn terrain_produces_terrain_and_water() {
        let generator = TerrainGenerator::new(7, TerrainParams::default());
        let mut stone = false;
        let mut grass = false;
        let mut dirt = false;
        let mut water = false;
        for sz in -128..128 {
            for sx in -128..128 {
                let surface = generator.height(sx, sz);
                grass |= surface >= 64 && generator.block_at(sx, surface, sz) == block::GRASS;
                water |= surface < 62 && generator.block_at(sx, 62, sz) == block::WATER;
                dirt |= generator.block_at(sx, surface - 1, sz) == block::DIRT;
                stone |= generator.block_at(sx, surface - 4, sz) == block::STONE;
                if grass && water && dirt && stone {
                    return;
                }
            }
        }
        assert!(stone);
        assert!(grass);
        assert!(dirt);
        assert!(water);
    }

    #[test]
    fn terrain_produces_foliage_cutout() {
        let params = TerrainParams {
            foliage_density: 1.0,
            ..TerrainParams::default()
        };
        let generator = TerrainGenerator::new(7, params);
        let mut leaves = false;
        let mut log = false;
        for sz in -128..128 {
            for sx in -128..128 {
                let surface = generator.height(sx, sz);
                if surface < 64 {
                    continue;
                }
                let trunk_height = generator.trunk_height(sx, sz);
                leaves |= generator.block_at(sx, surface + trunk_height + 1, sz) == block::LEAVES;
                log |= generator.block_at(sx, surface + 1, sz) == block::LOG;
                if leaves && log {
                    return;
                }
            }
        }
        assert!(leaves);
        assert!(log);
    }

    fn region_hash(generator: &TerrainGenerator) -> u64 {
        let mut hash = 0xcbf29ce484222325u64;
        for sy in 2..6 {
            for sz in -2..2 {
                for sx in -2..2 {
                    let section = generator.generate_section(sx, sy, sz);
                    for index in 0..SECTION_SIZE {
                        let state = section.get(index);
                        hash ^= state as u64;
                        hash = hash.wrapping_mul(0x100000001b3);
                    }
                }
            }
        }
        hash
    }
}
