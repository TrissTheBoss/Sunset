pub struct IntervalRing {
    buf: Vec<f32>,
    capacity: usize,
    head: usize,
    len: usize,
}

impl IntervalRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: vec![0.0; capacity],
            capacity,
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, value: f32) {
        if self.capacity == 0 {
            return;
        }
        let index = (self.head + self.len) % self.capacity;
        self.buf[index] = value;
        if self.len < self.capacity {
            self.len += 1;
        } else {
            self.head = (self.head + 1) % self.capacity;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn values(&self) -> Vec<f32> {
        let mut values = Vec::with_capacity(self.len);
        for offset in 0..self.len {
            values.push(self.buf[(self.head + offset) % self.capacity]);
        }
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_keeps_most_recent_values() {
        let mut ring = IntervalRing::new(3);
        ring.push(1.0);
        ring.push(2.0);
        ring.push(3.0);
        ring.push(4.0);
        assert_eq!(ring.len(), 3);
        assert_eq!(ring.values(), vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn recording_intervals_allocates_nothing() {
        let mut ring = IntervalRing::new(1024);
        let (counts, ()) = alloc_counter::count_alloc(|| {
            for i in 0..1_000_000 {
                ring.push(i as f32);
            }
        });
        assert_eq!(counts, (0, 0, 0));
    }
}
