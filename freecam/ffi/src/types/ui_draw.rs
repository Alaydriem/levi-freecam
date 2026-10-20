use freecam_core::structs::UiDraw;

/// One draw call: `index_count` indices from `index_offset`, each plus `vertex_offset`, with `texture` bound and the
/// scissor set to `clip`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcUiDraw {
    /// Physical pixels: min x, min y, max x, max y
    pub clip: [f32; 4],
    pub texture: u64,
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
}

impl From<&UiDraw> for FcUiDraw {
    fn from(value: &UiDraw) -> Self {
        Self {
            clip: value.clip,
            texture: value.texture,
            index_offset: value.index_offset,
            index_count: value.index_count,
            vertex_offset: value.vertex_offset,
        }
    }
}
