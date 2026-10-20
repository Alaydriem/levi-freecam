use freecam_core::structs::UiVertex;

/// One UI vertex, 20 bytes: position float2, uv float2, colour R8G8B8A8_UNORM (premultiplied alpha, gamma space).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcUiVertex {
    /// Physical pixels
    pub x: f32,
    /// Physical pixels
    pub y: f32,
    pub u: f32,
    pub v: f32,
    pub rgba: [u8; 4],
}

impl From<&UiVertex> for FcUiVertex {
    fn from(value: &UiVertex) -> Self {
        Self { x: value.x, y: value.y, u: value.u, v: value.v, rgba: value.rgba }
    }
}
