use std::sync::mpsc;

pub struct GpuTimer {
    query_set: wgpu::QuerySet,
    resolve_buffers: [wgpu::Buffer; 3],
    readback_buffers: [wgpu::Buffer; 3],
    timestamp_period_ns: f32,
    frame: usize,
    duration_ms: f32,
}

impl GpuTimer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            ty: wgpu::QueryType::Timestamp,
            count: 6,
        });
        let resolve_buffers = std::array::from_fn(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 16,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
                mapped_at_creation: false,
            })
        });
        let readback_buffers = std::array::from_fn(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 16,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        Self {
            query_set,
            resolve_buffers,
            readback_buffers,
            timestamp_period_ns: queue.get_timestamp_period(),
            frame: 0,
            duration_ms: 0.0,
        }
    }

    pub fn timestamp_writes(&self) -> wgpu::RenderPassTimestampWrites<'_> {
        let base = (self.frame % 3) as u32 * 2;
        wgpu::RenderPassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(base),
            end_of_pass_write_index: Some(base + 1),
        }
    }

    pub fn resolve(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let slot = self.frame % 3;
        encoder.resolve_query_set(
            &self.query_set,
            (slot * 2) as u32..(slot * 2 + 2) as u32,
            &self.resolve_buffers[slot],
            0,
        );
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffers[slot],
            0,
            &self.readback_buffers[slot],
            0,
            16,
        );
    }

    pub fn readback_latest(&mut self, device: &wgpu::Device) {
        if self.frame == 0 {
            return;
        }
        let slot = (self.frame - 1) % 3;
        let period = self.timestamp_period_ns;
        let mut duration_ms = 0.0;
        {
            let buffer = &self.readback_buffers[slot];
            let (sender, receiver) = mpsc::channel();
            buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| {
                    let _ = sender.send(result);
                });
            let _ = device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            });
            if let Ok(Ok(())) = receiver.recv() {
                if let Ok(view) = buffer.slice(..).get_mapped_range() {
                    let begin = u64::from_le_bytes(view[0..8].try_into().unwrap());
                    let end = u64::from_le_bytes(view[8..16].try_into().unwrap());
                    let ticks = end.saturating_sub(begin);
                    duration_ms = ticks as f32 * period / 1_000_000.0;
                }
                buffer.unmap();
            }
        }
        self.duration_ms = duration_ms;
    }

    pub fn duration_ms(&self) -> f32 {
        self.duration_ms
    }

    pub fn advance(&mut self) {
        self.frame += 1;
    }
}
