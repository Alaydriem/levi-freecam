use crate::types::{FcUiDraw, FcUiTextureDelta, FcUiVertex};

/// One UI frame. Apply `textures` in array order before drawing, draw `draws` in order, and release the `freed`
/// texture ids after drawing. When one frame changes more than one texture, the order between different texture ids
/// is not fixed; deltas for the same id keep their order. Every pointer is valid for its count and borrows buffers the
/// handle owns (see fc_ui_frame for how long).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FcUiFrame {
    pub vertices: *const FcUiVertex,
    pub vertex_count: u32,
    pub indices: *const u32,
    pub index_count: u32,
    pub draws: *const FcUiDraw,
    pub draw_count: u32,
    pub textures: *const FcUiTextureDelta,
    pub texture_count: u32,
    pub freed: *const u64,
    pub freed_count: u32,
    /// 1: the pointer is over the UI, or the UI is using it. Information only: the handle already hands it to core,
    /// where a right press over the UI starts no look drag. The shim passes mouse messages on to the game, and core
    /// keeps game clicks out through fc_mouse_button while freecam is on
    pub wants_pointer: u8,
    /// 0 default arrow, 1 hand
    pub cursor: u8,
    /// 1: over ui.maxVertices, no geometry; textures are still filled
    pub skipped: u8,
    /// 1: time the UI draw on the GPU and pass it as gpu_ms next frame
    pub measure_gpu: u8,
}

impl Default for FcUiFrame {
    fn default() -> Self {
        Self {
            vertices: std::ptr::null(),
            vertex_count: 0,
            indices: std::ptr::null(),
            index_count: 0,
            draws: std::ptr::null(),
            draw_count: 0,
            textures: std::ptr::null(),
            texture_count: 0,
            freed: std::ptr::null(),
            freed_count: 0,
            wants_pointer: 0,
            cursor: 0,
            skipped: 0,
            measure_gpu: 0,
        }
    }
}
