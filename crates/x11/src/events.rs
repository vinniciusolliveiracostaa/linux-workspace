use macrde_core::Rectangle;

pub enum X11Event {
    Expose { window_id: u32 },
    KeyPress { keycode: u8, modifiers: u16 },
    MapRequest { window_id: u32 },
    Unmap { window_id: u32 },
    Destroy { window_id: u32 },
    ConfigureRequest { window_id: u32, geometry: Rectangle },
    Unknown,
}
