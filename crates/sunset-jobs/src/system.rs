use crate::queue::PendingQueue;
use crate::{Job, JobResult};
use std::sync::{mpsc, Arc, Condvar, Mutex};

pub struct JobSystem<T: Send + 'static> {
    shared: Arc<Shared<T>>,
    workers: Vec<std::thread::JoinHandle<()>>,
    result_rx: mpsc::Receiver<JobResult<T>>,
}

struct Shared<T> {
    mutex: Mutex<Inner<T>>,
    condvar: Condvar,
}

struct Inner<T> {
    pending: PendingQueue<T>,
    shutdown: bool,
}

impl<T: Send + 'static> JobSystem<T> {
    pub fn new(worker_count: usize, queue_capacity: usize, result_capacity: usize) -> Self {
        assert!(worker_count > 0);
        let shared = Arc::new(Shared {
            mutex: Mutex::new(Inner {
                pending: PendingQueue::new(queue_capacity),
                shutdown: false,
            }),
            condvar: Condvar::new(),
        });
        let (result_tx, result_rx) = mpsc::sync_channel(result_capacity);
        let mut workers = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let shared = Arc::clone(&shared);
            let result_tx = result_tx.clone();
            workers.push(std::thread::spawn(move || worker_loop(shared, result_tx)));
        }
        Self {
            shared,
            workers,
            result_rx,
        }
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn submit(&self, job: Job<T>) -> Result<(), SubmitError> {
        let mut guard = self.shared.mutex.lock().unwrap();
        if guard.shutdown {
            return Err(SubmitError::Shutdown);
        }
        guard
            .pending
            .enqueue(job)
            .map_err(|_| SubmitError::QueueFull)?;
        self.shared.condvar.notify_one();
        Ok(())
    }

    pub fn cancel(&self, key: u64) {
        let mut guard = self.shared.mutex.lock().unwrap();
        guard.pending.cancel(key);
    }

    pub fn try_recv(&self) -> Option<JobResult<T>> {
        self.result_rx.try_recv().ok()
    }

    pub fn recv(&self) -> Option<JobResult<T>> {
        self.result_rx.recv().ok()
    }
}

impl<T: Send + 'static> Drop for JobSystem<T> {
    fn drop(&mut self) {
        if self.workers.is_empty() {
            return;
        }
        {
            let mut guard = self.shared.mutex.lock().unwrap();
            guard.shutdown = true;
            self.shared.condvar.notify_all();
        }
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmitError {
    QueueFull,
    Shutdown,
}

fn worker_loop<T: Send + 'static>(
    shared: Arc<Shared<T>>,
    result_tx: mpsc::SyncSender<JobResult<T>>,
) {
    loop {
        let job = {
            let mut guard = shared.mutex.lock().unwrap();
            loop {
                if guard.shutdown {
                    return;
                }
                if let Some(job) = guard.pending.dequeue() {
                    break job;
                }
                guard = shared.condvar.wait(guard).unwrap();
            }
        };
        let value = (job.work)();
        let _ = result_tx.send(JobResult {
            key: job.key,
            revision: job.revision,
            value,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Job;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};

    #[test]
    fn runs_jobs_and_delivers_results() {
        let system = JobSystem::new(2, 16, 16);
        system.submit(Job::new(1, 0, 0, || 41 + 1)).unwrap();
        let result = system.recv().unwrap();
        assert_eq!(result.key, 1);
        assert_eq!(result.revision, 0);
        assert_eq!(result.value, 42);
    }

    #[test]
    fn cancel_removes_pending_job_before_it_runs() {
        let system = JobSystem::new(1, 16, 16);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        system
            .submit(Job::new(1, 0, 0, move || {
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                1
            }))
            .unwrap();

        started_rx.recv().unwrap();

        let b_ran = Arc::new(AtomicBool::new(false));
        let c_ran = Arc::new(AtomicBool::new(false));
        let b_flag = Arc::clone(&b_ran);
        let c_flag = Arc::clone(&c_ran);
        system
            .submit(Job::new(2, 0, 0, move || {
                b_flag.store(true, Ordering::SeqCst);
                2
            }))
            .unwrap();
        system
            .submit(Job::new(3, 0, 0, move || {
                c_flag.store(true, Ordering::SeqCst);
                3
            }))
            .unwrap();
        system.cancel(3);

        release_tx.send(()).unwrap();
        assert_eq!(system.recv().unwrap().key, 1);
        assert_eq!(system.recv().unwrap().key, 2);
        assert!(b_ran.load(Ordering::SeqCst));
        assert!(!c_ran.load(Ordering::SeqCst));
        assert!(system.try_recv().is_none());
    }

    #[test]
    fn stale_result_is_flagged_by_revision() {
        let system = JobSystem::new(1, 16, 16);
        system.submit(Job::new(7, 1, 0, || 100)).unwrap();
        let result = system.recv().unwrap();
        assert!(!result.is_stale(1));
        assert!(result.is_stale(2));
    }

    #[test]
    fn submit_rejects_when_queue_is_full() {
        let system = JobSystem::new(1, 2, 16);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        system
            .submit(Job::new(0, 0, 0, move || {
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                0
            }))
            .unwrap();
        started_rx.recv().unwrap();

        assert!(system.submit(Job::new(1, 0, 0, || 1)).is_ok());
        assert!(system.submit(Job::new(2, 0, 0, || 2)).is_ok());
        assert_eq!(
            system.submit(Job::new(3, 0, 0, || 3)),
            Err(SubmitError::QueueFull)
        );

        release_tx.send(()).unwrap();
        let mut keys = vec![system.recv().unwrap().key, system.recv().unwrap().key];
        keys.push(system.recv().unwrap().key);
        keys.sort_unstable();
        assert_eq!(keys, vec![0, 1, 2]);
    }
}
