// one RGBA8 texture upload; the pixels are `byte_len` bytes at `byte_offset` in `UiFrameOutput::texture_bytes`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiTextureDelta {
    pub texture: u64,
    // true: create the texture again at this size; false: update the box at x, y
    pub full: bool,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub byte_offset: usize,
    pub byte_len: usize,
}
