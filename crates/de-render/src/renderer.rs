use crate::buffer::FrameBuffer;
use de_core::{Color, Rectangle};
use tiny_skia::{FillRule, Paint, PathBuilder, Transform};

pub struct SoftwareRenderer {
    buffer: FrameBuffer,
}

impl SoftwareRenderer {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let buffer = FrameBuffer::new(width, height)?;
        Some(Self { buffer })
    }

    pub fn clear(&mut self, color: Color) {
        let pixmap = self.buffer.pixmap_mut();
        pixmap.fill(tiny_skia::Color::from_rgba8(
            color.r, color.g, color.b, color.a,
        ));
    }

    pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color, corner_radius: f32) {
        let mut paint = Paint::default();
        paint.set_color(tiny_skia::Color::from_rgba8(
            color.r, color.g, color.b, color.a,
        ));
        paint.anti_alias = true;

        let path = if corner_radius > 0.0 {
            self.rounded_rect_path(rect, corner_radius)
        } else {
            self.rect_path(rect)
        };

        if let Some(path) = path {
            let pixmap = self.buffer.pixmap_mut();
            pixmap.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    pub fn get_buffer(&self) -> &[u8] {
        self.buffer.data()
    }

    pub fn width(&self) -> u32 {
        self.buffer.width()
    }

    pub fn height(&self) -> u32 {
        self.buffer.height()
    }

    fn rect_path(&self, rect: Rectangle) -> Option<tiny_skia::Path> {
        let ts_rect = tiny_skia::Rect::from_xywh(
            rect.position.x as f32,
            rect.position.y as f32,
            rect.size.width as f32,
            rect.size.height as f32,
        )?;

        Some(PathBuilder::from_rect(ts_rect))
    }

    fn rounded_rect_path(&self, rect: Rectangle, _radius: f32) -> Option<tiny_skia::Path> {
        let ts_rect = tiny_skia::Rect::from_xywh(
            rect.position.x as f32,
            rect.position.y as f32,
            rect.size.width as f32,
            rect.size.height as f32,
        )?;

        // tiny-skia não tem rounded rect nativo
        // Por enquanto, retorna retângulo normal (implementar rounded rect depois)
        Some(PathBuilder::from_rect(ts_rect))
    }
}
