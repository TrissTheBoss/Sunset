use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuSpan {
    InputEvents,
    SnapshotAcquire,
    SectionVisibility,
    EntitiesPrepare,
    HudBuild,
    UploadSchedule,
    CommandEncode,
    AcquirePresent,
}

pub const CPU_SPAN_COUNT: usize = 8;

pub struct CpuTrace {
    frame_us: [f64; CPU_SPAN_COUNT],
    active: Option<(CpuSpan, Instant)>,
}

impl Default for CpuTrace {
    fn default() -> Self {
        Self {
            frame_us: [0.0; CPU_SPAN_COUNT],
            active: None,
        }
    }
}

impl CpuTrace {
    pub fn begin(&mut self, span: CpuSpan) {
        self.active = Some((span, Instant::now()));
    }

    pub fn end(&mut self) {
        if let Some((span, start)) = self.active.take() {
            self.frame_us[span.index()] += start.elapsed().as_secs_f64() * 1_000_000.0;
        }
    }

    pub fn reset(&mut self) {
        self.frame_us = [0.0; CPU_SPAN_COUNT];
        self.active = None;
    }

    pub fn frame_us(&self, span: CpuSpan) -> f64 {
        self.frame_us[span.index()]
    }
}

impl CpuSpan {
    pub fn index(self) -> usize {
        match self {
            CpuSpan::InputEvents => 0,
            CpuSpan::SnapshotAcquire => 1,
            CpuSpan::SectionVisibility => 2,
            CpuSpan::EntitiesPrepare => 3,
            CpuSpan::HudBuild => 4,
            CpuSpan::UploadSchedule => 5,
            CpuSpan::CommandEncode => 6,
            CpuSpan::AcquirePresent => 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_indices_cover_the_full_array() {
        let spans = [
            CpuSpan::InputEvents,
            CpuSpan::SnapshotAcquire,
            CpuSpan::SectionVisibility,
            CpuSpan::EntitiesPrepare,
            CpuSpan::HudBuild,
            CpuSpan::UploadSchedule,
            CpuSpan::CommandEncode,
            CpuSpan::AcquirePresent,
        ];
        let mut seen = [false; CPU_SPAN_COUNT];
        for span in spans {
            seen[span.index()] = true;
        }
        assert!(seen.iter().all(|value| *value));
    }

    #[test]
    fn accumulates_split_spans_and_resets() {
        let mut trace = CpuTrace::default();
        trace.begin(CpuSpan::AcquirePresent);
        std::thread::sleep(std::time::Duration::from_millis(1));
        trace.end();
        trace.begin(CpuSpan::AcquirePresent);
        std::thread::sleep(std::time::Duration::from_millis(1));
        trace.end();
        assert!(trace.frame_us(CpuSpan::AcquirePresent) >= 2_000.0);
        trace.reset();
        assert_eq!(trace.frame_us(CpuSpan::AcquirePresent), 0.0);
    }
}
