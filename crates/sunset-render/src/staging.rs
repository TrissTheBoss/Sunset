use crate::allocator::{Range, RangeAllocator};

pub const DEFAULT_UPLOAD_BUDGET_BYTES: u64 = 2 * 1024 * 1024;
pub const DEFAULT_UPLOAD_BUDGET_US: u64 = 25;

#[derive(Clone, Copy, Debug)]
pub struct UploadBudget {
    pub bytes: u64,
    pub cpu_micros: u64,
}

impl Default for UploadBudget {
    fn default() -> Self {
        Self {
            bytes: DEFAULT_UPLOAD_BUDGET_BYTES,
            cpu_micros: DEFAULT_UPLOAD_BUDGET_US,
        }
    }
}

pub struct StagingRing {
    frames_in_flight: u32,
    slots: Vec<RangeAllocator>,
    owner_frame: Vec<Option<u64>>,
}

impl StagingRing {
    pub fn new(slot_count: u32, capacity_per_slot: u64, frames_in_flight: u32) -> Self {
        debug_assert!(slot_count > 0);
        debug_assert!(frames_in_flight > 0);
        debug_assert!(slot_count >= frames_in_flight);
        Self {
            frames_in_flight,
            slots: (0..slot_count)
                .map(|_| RangeAllocator::new(capacity_per_slot))
                .collect(),
            owner_frame: vec![None; slot_count as usize],
        }
    }

    pub fn begin_frame(&mut self, frame: u64) {
        let index = self.slot_index(frame);
        if let Some(previous) = self.owner_frame[index] {
            debug_assert!(
                frame.saturating_sub(previous) >= self.frames_in_flight as u64,
                "staging slot {index} reused while frame {previous} is still in flight"
            );
        }
        self.owner_frame[index] = Some(frame);
        self.slots[index].reset();
    }

    pub fn allocate(&mut self, frame: u64, size: u64) -> Option<Range> {
        let index = self.slot_index(frame);
        self.slots[index].allocate(size, 1)
    }

    fn slot_index(&self, frame: u64) -> usize {
        (frame % self.slots.len() as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_reused_after_in_flight_window() {
        let mut ring = StagingRing::new(2, 100, 2);
        ring.begin_frame(0);
        assert!(ring.allocate(0, 100).is_some());
        assert!(ring.allocate(0, 1).is_none());
        ring.begin_frame(1);
        assert!(ring.allocate(1, 100).is_some());
        ring.begin_frame(2);
        assert!(ring.allocate(2, 100).is_some());
    }

    #[test]
    #[should_panic(expected = "still in flight")]
    fn reusing_in_flight_slot_panics() {
        let mut ring = StagingRing::new(2, 100, 2);
        ring.begin_frame(0);
        ring.begin_frame(0);
    }
}
