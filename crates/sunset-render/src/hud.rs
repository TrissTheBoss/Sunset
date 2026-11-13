use crate::font::{glyph_rows, GLYPH_ADVANCE, GLYPH_HEIGHT, GLYPH_WIDTH};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HudVertex {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
    pub color: [u8; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudMaterial {
    Solid,
    Text,
}

struct HudQuad {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
}

struct QuadBuffer {
    vertices: Box<[HudVertex]>,
    indices: Box<[u32]>,
    vertex_count: usize,
    index_count: usize,
}

impl QuadBuffer {
    fn new(max_quads: usize) -> Self {
        Self {
            vertices: vec![HudVertex::default(); max_quads * 4].into_boxed_slice(),
            indices: vec![0u32; max_quads * 6].into_boxed_slice(),
            vertex_count: 0,
            index_count: 0,
        }
    }

    fn clear(&mut self) {
        self.vertex_count = 0;
        self.index_count = 0;
    }

    fn push_quad(&mut self, quad: HudQuad, color: [u8; 4]) {
        if self.vertex_count + 4 > self.vertices.len() {
            return;
        }
        let base = self.vertex_count;
        self.vertices[base] = HudVertex {
            x: quad.x0,
            y: quad.y0,
            u: quad.u0,
            v: quad.v0,
            color,
        };
        self.vertices[base + 1] = HudVertex {
            x: quad.x1,
            y: quad.y0,
            u: quad.u1,
            v: quad.v0,
            color,
        };
        self.vertices[base + 2] = HudVertex {
            x: quad.x1,
            y: quad.y1,
            u: quad.u1,
            v: quad.v1,
            color,
        };
        self.vertices[base + 3] = HudVertex {
            x: quad.x0,
            y: quad.y1,
            u: quad.u0,
            v: quad.v1,
            color,
        };
        let index_base = base as u32;
        self.indices[self.index_count] = index_base;
        self.indices[self.index_count + 1] = index_base + 1;
        self.indices[self.index_count + 2] = index_base + 2;
        self.indices[self.index_count + 3] = index_base;
        self.indices[self.index_count + 4] = index_base + 2;
        self.indices[self.index_count + 5] = index_base + 3;
        self.vertex_count += 4;
        self.index_count += 6;
    }
}

pub struct HudBatch {
    solid: QuadBuffer,
    text: QuadBuffer,
}

impl HudBatch {
    pub fn new(max_quads: usize) -> Self {
        Self {
            solid: QuadBuffer::new(max_quads),
            text: QuadBuffer::new(max_quads),
        }
    }

    pub fn clear(&mut self) {
        self.solid.clear();
        self.text.clear();
    }

    pub fn push_rect(
        &mut self,
        material: HudMaterial,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [u8; 4],
    ) {
        let buffer = match material {
            HudMaterial::Solid => &mut self.solid,
            HudMaterial::Text => &mut self.text,
        };
        buffer.push_quad(
            HudQuad {
                x0: x,
                y0: y,
                x1: x + w,
                y1: y + h,
                u0: 0.0,
                v0: 0.0,
                u1: 1.0,
                v1: 1.0,
            },
            color,
        );
    }

    pub fn push_text(&mut self, text: &str, x: f32, y: f32, scale: f32, color: [u8; 4]) {
        let glyph_width = GLYPH_WIDTH as f32 * scale;
        let glyph_height = GLYPH_HEIGHT as f32 * scale;
        let advance = GLYPH_ADVANCE as f32 * scale;
        let mut cursor = x;
        for ch in text.chars() {
            if glyph_rows(ch).is_some() {
                self.text.push_quad(
                    HudQuad {
                        x0: cursor,
                        y0: y,
                        x1: cursor + glyph_width,
                        y1: y + glyph_height,
                        u0: 0.0,
                        v0: 0.0,
                        u1: 1.0,
                        v1: 1.0,
                    },
                    color,
                );
            }
            cursor += advance;
        }
    }

    pub fn draw_count(&self) -> usize {
        (self.solid.index_count > 0) as usize + (self.text.index_count > 0) as usize
    }

    pub fn solid_vertices(&self) -> &[HudVertex] {
        &self.solid.vertices[..self.solid.vertex_count]
    }

    pub fn solid_indices(&self) -> &[u32] {
        &self.solid.indices[..self.solid.index_count]
    }

    pub fn text_vertices(&self) -> &[HudVertex] {
        &self.text.vertices[..self.text.vertex_count]
    }

    pub fn text_indices(&self) -> &[u32] {
        &self.text.indices[..self.text.index_count]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_draw_per_material_class() {
        let mut batch = HudBatch::new(64);
        batch.push_rect(HudMaterial::Solid, 0.0, 0.0, 100.0, 10.0, [255, 0, 0, 255]);
        batch.push_rect(HudMaterial::Solid, 0.0, 20.0, 100.0, 10.0, [0, 255, 0, 255]);
        batch.push_text("FPS: 60", 0.0, 40.0, 1.0, [255, 255, 255, 255]);
        assert_eq!(batch.draw_count(), 2);
        assert_eq!(batch.solid_indices().len(), 12);
        assert_eq!(batch.text_indices().len(), 6 * 7);
    }

    #[test]
    fn changing_text_reuses_fixed_buffers() {
        let mut batch = HudBatch::new(128);
        batch.push_text("12345", 0.0, 0.0, 1.0, [255, 255, 255, 255]);
        let first_vertices = batch.text_vertices().len();
        let first_indices = batch.text_indices().len();
        assert_eq!(first_vertices, 4 * 5);
        assert_eq!(first_indices, 6 * 5);

        batch.clear();
        batch.push_text("ABCDEFGH", 0.0, 0.0, 1.0, [255, 255, 255, 255]);
        assert_eq!(batch.text_vertices().len(), 4 * 8);
        assert_eq!(batch.text_indices().len(), 6 * 8);
        assert_eq!(batch.solid_vertices().len(), 0);
    }

    #[test]
    fn unsupported_characters_are_skipped_but_advance() {
        let mut batch = HudBatch::new(64);
        batch.push_text("A!B", 0.0, 0.0, 1.0, [255, 255, 255, 255]);
        assert_eq!(batch.text_vertices().len(), 4 * 2);
        assert_eq!(batch.text_indices().len(), 6 * 2);
    }
}
