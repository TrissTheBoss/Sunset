#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Range {
    pub offset: u64,
    pub size: u64,
}

impl Range {
    pub fn end(self) -> u64 {
        self.offset + self.size
    }
}

pub struct RangeAllocator {
    capacity: u64,
    free: Vec<Range>,
}

impl RangeAllocator {
    pub fn new(capacity: u64) -> Self {
        debug_assert!(capacity > 0);
        Self {
            capacity,
            free: vec![Range {
                offset: 0,
                size: capacity,
            }],
        }
    }

    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    pub fn allocate(&mut self, size: u64, alignment: u64) -> Option<Range> {
        debug_assert!(size > 0);
        debug_assert!(alignment > 0 && alignment.is_power_of_two());
        let mut selected = None;
        for (index, block) in self.free.iter().enumerate() {
            let aligned = align_up(block.offset, alignment);
            if aligned.saturating_add(size) <= block.end() {
                selected = Some(index);
                break;
            }
        }
        let index = selected?;
        let block = self.free.remove(index);
        let allocated = Range {
            offset: align_up(block.offset, alignment),
            size,
        };
        let mut insert_at = index;
        if allocated.offset > block.offset {
            self.free.insert(
                insert_at,
                Range {
                    offset: block.offset,
                    size: allocated.offset - block.offset,
                },
            );
            insert_at += 1;
        }
        let allocated_end = allocated.end();
        if allocated_end < block.end() {
            self.free.insert(
                insert_at,
                Range {
                    offset: allocated_end,
                    size: block.end() - allocated_end,
                },
            );
        }
        Some(allocated)
    }

    pub fn free(&mut self, range: Range) {
        debug_assert!(range.size > 0);
        debug_assert!(range.end() <= self.capacity);
        debug_assert!(
            !self
                .free
                .iter()
                .any(|block| block.offset < range.end() && range.offset < block.end()),
            "freeing an overlapping range"
        );
        self.free.push(range);
        self.free.sort_by_key(|block| block.offset);
        self.coalesce();
    }

    pub fn reset(&mut self) {
        self.free.clear();
        self.free.push(Range {
            offset: 0,
            size: self.capacity,
        });
    }

    pub fn free_bytes(&self) -> u64 {
        self.free.iter().map(|block| block.size).sum()
    }

    fn coalesce(&mut self) {
        let mut merged: Vec<Range> = Vec::with_capacity(self.free.len());
        for block in &self.free {
            match merged.last_mut() {
                Some(last) if last.end() == block.offset => last.size += block.size,
                _ => merged.push(*block),
            }
        }
        self.free = merged;
    }
}

fn align_up(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_within_capacity_and_reuses() {
        let mut allocator = RangeAllocator::new(1024);
        let a = allocator.allocate(256, 1).unwrap();
        let b = allocator.allocate(256, 1).unwrap();
        let c = allocator.allocate(512, 1).unwrap();
        assert_eq!(a.end(), b.offset);
        assert_eq!(b.end(), c.offset);
        assert_eq!(allocator.free_bytes(), 0);
        assert!(allocator.allocate(1, 1).is_none());

        allocator.free(b);
        assert_eq!(allocator.free_bytes(), 256);
        let d = allocator.allocate(256, 1).unwrap();
        assert_eq!(d, b);
        assert_eq!(allocator.free_bytes(), 0);

        allocator.free(a);
        allocator.free(c);
        allocator.free(d);
        assert_eq!(allocator.free_bytes(), 1024);
        assert_eq!(allocator.free.len(), 1);
    }

    #[test]
    fn aligns_allocations_to_requested_boundary() {
        let mut allocator = RangeAllocator::new(1024);
        allocator.allocate(10, 1).unwrap();
        let aligned = allocator.allocate(100, 256).unwrap();
        assert_eq!(aligned.offset % 256, 0);
        assert!(aligned.end() <= 1024);
    }

    #[test]
    fn property_no_overlap_and_full_reuse_accounting() {
        let mut state = 0x1234_5678_9abc_def0u64;
        for _ in 0..100 {
            let mut allocator = RangeAllocator::new(4096);
            let mut live: Vec<Range> = Vec::new();
            for _ in 0..200 {
                if live.is_empty() || !next_u64(&mut state).is_multiple_of(3) {
                    let size = 1 + next_u64(&mut state) % 256;
                    let alignment = [1, 4, 16, 64][(next_u64(&mut state) % 4) as usize];
                    if let Some(range) = allocator.allocate(size, alignment) {
                        assert_eq!(range.offset % alignment, 0);
                        assert!(range.end() <= 4096);
                        for existing in &live {
                            assert!(
                                range.offset >= existing.end() || existing.offset >= range.end(),
                                "overlap between {range:?} and {existing:?}"
                            );
                        }
                        live.push(range);
                    }
                } else {
                    let index = (next_u64(&mut state) as usize) % live.len();
                    allocator.free(live.swap_remove(index));
                }
                let used = live.iter().map(|range| range.size).sum::<u64>();
                assert_eq!(allocator.free_bytes() + used, 4096);
            }
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
