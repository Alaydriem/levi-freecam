// one draw call over a range of `UiFrameOutput::indices`
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UiDraw {
    // scissor rectangle in physical pixels: min x, min y, max x, max y
    pub clip: [f32; 4],
    pub texture: u64,
    pub index_offset: u32,
    pub index_count: u32,
    // added to every index of this draw
    pub vertex_offset: u32,
}
