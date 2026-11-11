#![forbid(unsafe_code)]

pub use alloc_counter::{count_alloc, AllocCounterSystem};

#[global_allocator]
static ALLOCATOR: AllocCounterSystem = AllocCounterSystem;

mod counters;
mod interval;
mod percentile;
mod throughput;

pub use counters::Counters;
pub use interval::IntervalRing;
pub use percentile::percentile;
pub use throughput::frames_per_second;
