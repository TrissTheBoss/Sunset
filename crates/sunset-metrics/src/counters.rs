#[derive(Default)]
pub struct Counters {
    draws: u64,
    passes: u64,
    submits: u64,
    resource_creations: u64,
    uploads: u64,
}

impl Counters {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn record_draw(&mut self) {
        self.draws += 1;
    }

    pub fn record_pass(&mut self) {
        self.passes += 1;
    }

    pub fn record_submit(&mut self) {
        self.submits += 1;
    }

    pub fn record_resource_creation(&mut self) {
        self.resource_creations += 1;
    }

    pub fn record_upload(&mut self) {
        self.uploads += 1;
    }

    pub fn draws(&self) -> u64 {
        self.draws
    }

    pub fn passes(&self) -> u64 {
        self.passes
    }

    pub fn submits(&self) -> u64 {
        self.submits
    }

    pub fn resource_creations(&self) -> u64 {
        self.resource_creations
    }

    pub fn uploads(&self) -> u64 {
        self.uploads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_accumulate_and_reset() {
        let mut counters = Counters::default();
        counters.record_draw();
        counters.record_draw();
        counters.record_pass();
        assert_eq!(counters.draws(), 2);
        assert_eq!(counters.passes(), 1);
        assert_eq!(counters.submits(), 0);
        counters.reset();
        assert_eq!(counters.draws(), 0);
    }
}
