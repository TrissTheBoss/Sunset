use crate::allocator::{Range, RangeAllocator};

pub struct GeometryArena {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_allocator: RangeAllocator,
    index_allocator: RangeAllocator,
}

impl GeometryArena {
    pub fn new(device: &wgpu::Device, vertex_capacity: u64, index_capacity: u64) -> Self {
        debug_assert!(vertex_capacity > 0);
        debug_assert!(index_capacity > 0);
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: vertex_capacity,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: index_capacity,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            vertex_buffer,
            index_buffer,
            vertex_allocator: RangeAllocator::new(vertex_capacity),
            index_allocator: RangeAllocator::new(index_capacity),
        }
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn allocate_vertices(&mut self, size: u64, alignment: u64) -> Option<Range> {
        self.vertex_allocator.allocate(size, alignment)
    }

    pub fn allocate_indices(&mut self, size: u64, alignment: u64) -> Option<Range> {
        self.index_allocator.allocate(size, alignment)
    }

    pub fn free_vertices(&mut self, range: Range) {
        self.vertex_allocator.free(range);
    }

    pub fn free_indices(&mut self, range: Range) {
        self.index_allocator.free(range);
    }
}
