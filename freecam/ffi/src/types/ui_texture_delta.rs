use freecam_core::structs::UiTextureDelta;

/// One RGBA8 UNORM texture upload of `width` x `height` pixels: `byte_len` bytes at `rgba`, rows packed with no
/// padding.
///
/// `full` 1: create the texture at this size. When the id is already held, release the old texture and create it
/// again: after fc_ui_reset_textures the font texture comes again in full with the same id. `full` 0: update the box
/// at `x`, `y` of a texture already held.
///
/// `rgba` is null and `byte_len` 0 only when the bytes are missing, which the UI runtime does not produce; skip such a
/// delta.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FcUiTextureDelta {
    pub texture: u64,
    pub full: u8,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub rgba: *const u8,
    pub byte_len: usize,
}

impl FcUiTextureDelta {
    // the delta with rgba pointing at its pixels inside `bytes`, the frame's byte buffer
    pub fn new(delta: &UiTextureDelta, bytes: &[u8]) -> Self {
        let pixels = delta.byte_offset.checked_add(delta.byte_len).and_then(|end| bytes.get(delta.byte_offset..end));
        Self {
            texture: delta.texture,
            full: u8::from(delta.full),
            x: delta.x,
            y: delta.y,
            width: delta.width,
            height: delta.height,
            rgba: pixels.map_or(std::ptr::null(), <[u8]>::as_ptr),
            byte_len: pixels.map_or(0, <[u8]>::len),
        }
    }
}
