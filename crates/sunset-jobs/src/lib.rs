#![forbid(unsafe_code)]

mod queue;
mod revision;
mod system;

pub use queue::PendingQueue;
pub use revision::RevisionTable;
pub use system::{JobSystem, SubmitError};

pub type Priority = u32;

pub struct Job<T> {
    pub key: u64,
    pub revision: u64,
    pub priority: Priority,
    pub work: Box<dyn FnOnce() -> T + Send + 'static>,
}

impl<T> std::fmt::Debug for Job<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Job")
            .field("key", &self.key)
            .field("revision", &self.revision)
            .field("priority", &self.priority)
            .finish_non_exhaustive()
    }
}

impl<T> Job<T> {
    pub fn new(
        key: u64,
        revision: u64,
        priority: Priority,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Self {
        Self {
            key,
            revision,
            priority,
            work: Box::new(work),
        }
    }
}

#[derive(Debug)]
pub struct JobResult<T> {
    pub key: u64,
    pub revision: u64,
    pub value: T,
}

impl<T> JobResult<T> {
    pub fn is_stale(&self, current_revision: u64) -> bool {
        self.revision < current_revision
    }
}

pub fn worker_count() -> usize {
    std::thread::available_parallelism()
        .map_or(1, usize::from)
        .saturating_sub(2)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_count_is_at_least_one() {
        assert!(worker_count() >= 1);
    }

    #[test]
    fn stale_result_is_detected_by_revision() {
        let result = JobResult {
            key: 0,
            revision: 3,
            value: (),
        };
        assert!(!result.is_stale(3));
        assert!(result.is_stale(4));
    }
}
