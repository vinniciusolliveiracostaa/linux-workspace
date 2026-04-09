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
            rect.size.width() as f32,
            rect.size.height() as f32,
        )?;

        Some(PathBuilder::from_rect(ts_rect))
    }

    pub fn get_buffer_bgra(&self) -> Vec<u8> {
        self.buffer
            .data()
            .chunks_exact(4)
            .flat_map(|px| [px[2], px[1], px[0], px[3]]) // RGBA -> BGRA
            .collect()
    }

    fn rounded_rect_path(&self, rect: Rectangle, radius: f32) -> Option<tiny_skia::Path> {
        // POR QUE usar PathBuilder ao invés de PathBuilder::from_rect()?
        // - tiny-skia não tem rounded rect nativo
        // - Precisamos construir o path manualmente com Bézier curves
        // - quad_to() cria arcos suaves nos cantos

        let mut pb = PathBuilder::new();

        let x = rect.position.x as f32;
        let y = rect.position.y as f32;
        let w = rect.size.width() as f32;
        let h = rect.size.height() as f32;

        // Clamp radius para não exceder metade da menor dimensão
        // POR QUE? Se radius > w/2 ou radius > h/2, os arcos se sobrepõem
        let r = radius.min(w / 2.0).min(h / 2.0);

        // Se radius é 0, retornar retângulo normal (otimização)
        if r <= 0.0 {
            let ts_rect = tiny_skia::Rect::from_xywh(x, y, w, h)?;
            return Some(PathBuilder::from_rect(ts_rect));
        }

        // Construir path com cantos arredondados
        // Ordem: top-left → top-right → bottom-right → bottom-left

        // Começar no meio da borda superior (após o canto top-left)
        pb.move_to(x + r, y);

        // Top edge + top-right corner
        pb.line_to(x + w - r, y);
        pb.quad_to(x + w, y, x + w, y + r);

        // Right edge + bottom-right corner
        pb.line_to(x + w, y + h - r);
        pb.quad_to(x + w, y + h, x + w - r, y + h);

        // Bottom edge + bottom-left corner
        pb.line_to(x + r, y + h);
        pb.quad_to(x, y + h, x, y + h - r);

        // Left edge + top-left corner
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);

        // Fechar o path
        pb.close();
        pb.finish()
    }
}
