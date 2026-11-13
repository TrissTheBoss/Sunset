use crate::{Job, Priority};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

struct Queued<T> {
    priority: Priority,
    sequence: u64,
    job: Job<T>,
}

impl<T> PartialEq for Queued<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl<T> Eq for Queued<T> {}

impl<T> PartialOrd for Queued<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Queued<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

pub struct PendingQueue<T> {
    capacity: usize,
    heap: BinaryHeap<Queued<T>>,
    next_sequence: u64,
}

impl<T> PendingQueue<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            capacity,
            heap: BinaryHeap::new(),
            next_sequence: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn enqueue(&mut self, job: Job<T>) -> Result<(), Job<T>> {
        if self.heap.len() >= self.capacity {
            return Err(job);
        }
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        self.heap.push(Queued {
            priority: job.priority,
            sequence,
            job,
        });
        Ok(())
    }

    pub fn dequeue(&mut self) -> Option<Job<T>> {
        self.heap.pop().map(|queued| queued.job)
    }

    pub fn cancel(&mut self, key: u64) -> usize {
        let before = self.heap.len();
        let mut retained = BinaryHeap::new();
        for queued in self.heap.drain() {
            if queued.job.key != key {
                retained.push(queued);
            }
        }
        self.heap = retained;
        before - self.heap.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(key: u64, priority: Priority) -> Job<u64> {
        Job::new(key, 0, priority, move || key)
    }

    #[test]
    fn dequeues_highest_priority_first() {
        let mut queue = PendingQueue::new(8);
        queue.enqueue(job(1, 1)).unwrap();
        queue.enqueue(job(2, 3)).unwrap();
        queue.enqueue(job(3, 2)).unwrap();
        assert_eq!(queue.dequeue().unwrap().key, 2);
        assert_eq!(queue.dequeue().unwrap().key, 3);
        assert_eq!(queue.dequeue().unwrap().key, 1);
    }

    #[test]
    fn equal_priority_is_fifo() {
        let mut queue = PendingQueue::new(8);
        queue.enqueue(job(1, 5)).unwrap();
        queue.enqueue(job(2, 5)).unwrap();
        queue.enqueue(job(3, 5)).unwrap();
        assert_eq!(queue.dequeue().unwrap().key, 1);
        assert_eq!(queue.dequeue().unwrap().key, 2);
        assert_eq!(queue.dequeue().unwrap().key, 3);
    }

    #[test]
    fn rejects_when_full() {
        let mut queue = PendingQueue::new(2);
        assert!(queue.enqueue(job(1, 0)).is_ok());
        assert!(queue.enqueue(job(2, 0)).is_ok());
        assert!(queue.enqueue(job(3, 0)).is_err());
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn flood_never_exceeds_capacity() {
        let mut queue = PendingQueue::new(4);
        for key in 0..100u64 {
            let _ = queue.enqueue(job(key, 0));
        }
        assert_eq!(queue.len(), 4);
        assert!(queue.len() <= queue.capacity());
    }

    #[test]
    fn cancel_removes_all_jobs_for_key() {
        let mut queue = PendingQueue::new(8);
        queue.enqueue(job(1, 0)).unwrap();
        queue.enqueue(job(2, 0)).unwrap();
        queue.enqueue(job(1, 1)).unwrap();
        assert_eq!(queue.cancel(1), 2);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.dequeue().unwrap().key, 2);
    }
}
