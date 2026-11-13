pub const SECTION_SIZE: usize = 4096;

const MIN_BITS: u8 = 4;
const DIRECT_BITS: u8 = 16;

pub struct PalettedSection {
    palette: Vec<u16>,
    bits: u8,
    data: Vec<u64>,
}

impl PalettedSection {
    pub fn new(default: u16) -> Self {
        let palette = vec![default];
        Self {
            palette,
            bits: MIN_BITS,
            data: vec![0; words(MIN_BITS)],
        }
    }

    pub fn get(&self, index: usize) -> u16 {
        let entry = read_entry(&self.data, self.bits, index);
        if self.bits == DIRECT_BITS {
            entry as u16
        } else {
            self.palette[entry as usize]
        }
    }

    pub fn set(&mut self, index: usize, state: u16) {
        if self.bits == DIRECT_BITS {
            write_entry(&mut self.data, self.bits, index, state as u32);
            return;
        }
        if let Some(palette_index) = self.palette.iter().position(|entry| *entry == state) {
            write_entry(&mut self.data, self.bits, index, palette_index as u32);
            return;
        }
        if self.palette.len() < (1 << self.bits) {
            let palette_index = self.palette.len();
            self.palette.push(state);
            write_entry(&mut self.data, self.bits, index, palette_index as u32);
            return;
        }
        self.grow();
        self.set(index, state);
    }

    pub fn bits(&self) -> u8 {
        self.bits
    }

    pub fn palette_len(&self) -> usize {
        self.palette.len()
    }

    fn grow(&mut self) {
        let old_bits = self.bits;
        let new_bits = old_bits * 2;
        let old_data = std::mem::take(&mut self.data);
        self.bits = new_bits;
        self.data = vec![0; words(new_bits)];
        for index in 0..SECTION_SIZE {
            let entry = read_entry(&old_data, old_bits, index);
            if new_bits == DIRECT_BITS {
                let state = self.palette[entry as usize];
                write_entry(&mut self.data, new_bits, index, state as u32);
            } else {
                write_entry(&mut self.data, new_bits, index, entry);
            }
        }
        if new_bits == DIRECT_BITS {
            self.palette.clear();
        }
    }
}

fn words(bits: u8) -> usize {
    (SECTION_SIZE * bits as usize).div_ceil(64)
}

fn read_entry(data: &[u64], bits: u8, index: usize) -> u32 {
    let bit_offset = index * bits as usize;
    let word = bit_offset / 64;
    let bit_in_word = bit_offset % 64;
    let mask = (1u64 << bits) - 1;
    ((data[word] >> bit_in_word) & mask) as u32
}

fn write_entry(data: &mut [u64], bits: u8, index: usize, value: u32) {
    let bit_offset = index * bits as usize;
    let word = bit_offset / 64;
    let bit_in_word = bit_offset % 64;
    let mask = (1u64 << bits) - 1;
    data[word] = (data[word] & !(mask << bit_in_word)) | ((value as u64) << bit_in_word);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_single_value() {
        let section = PalettedSection::new(7);
        assert_eq!(section.bits(), 4);
        assert_eq!(section.palette_len(), 1);
        for index in [0, 1, 255, 256, 4095] {
            assert_eq!(section.get(index), 7);
        }
    }

    #[test]
    fn round_trip_with_edits() {
        let mut section = PalettedSection::new(0);
        let mut state = 1u64;
        for index in 0..SECTION_SIZE {
            let value = (next_u64(&mut state) % 8) as u16;
            section.set(index, value);
        }
        state = 1u64;
        for index in 0..SECTION_SIZE {
            let expected = (next_u64(&mut state) % 8) as u16;
            assert_eq!(section.get(index), expected);
        }
        assert_eq!(section.bits(), 4);
    }

    #[test]
    fn grows_to_eight_bits_when_palette_exceeds_sixteen() {
        let mut section = PalettedSection::new(0);
        for value in 0..17u16 {
            section.set(value as usize, value);
        }
        assert_eq!(section.bits(), 8);
        for value in 0..17u16 {
            assert_eq!(section.get(value as usize), value);
        }
    }

    #[test]
    fn grows_to_direct_bits_when_palette_exceeds_256() {
        let mut section = PalettedSection::new(0);
        for value in 0..257u16 {
            section.set(value as usize, value);
        }
        assert_eq!(section.bits(), 16);
        assert_eq!(section.palette_len(), 0);
        for value in 0..257u16 {
            assert_eq!(section.get(value as usize), value);
        }
    }

    #[test]
    fn property_round_trip_random_states() {
        let mut state = 0xd1342543de82ef95u64;
        let mut section = PalettedSection::new(0);
        let mut expected = vec![0u16; SECTION_SIZE];
        for _ in 0..20_000 {
            let index = next_u64(&mut state) as usize % SECTION_SIZE;
            let value = (next_u64(&mut state) % 1024) as u16;
            section.set(index, value);
            expected[index] = value;
        }
        for (index, &value) in expected.iter().enumerate() {
            assert_eq!(section.get(index), value);
        }
    }

    fn next_u64(state: &mut u64) -> u64 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        x
    }
}
