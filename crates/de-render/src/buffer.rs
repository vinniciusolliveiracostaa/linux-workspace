use tiny_skia::Pixmap;

pub struct FrameBuffer {
    pixmap: Pixmap,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let pixmap = Pixmap::new(width, height)?;
        Some(Self { pixmap })
    }

    pub fn pixmap_mut(&mut self) -> &mut Pixmap {
        &mut self.pixmap
    }

    pub fn data(&self) -> &[u8] {
        self.pixmap.data()
    }

    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }
}
