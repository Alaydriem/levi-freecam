use crate::structs::ui::{UiAction, UiCursor, UiDraw, UiTextureDelta, UiVertex};

// everything one UI frame produces; the buffers are reused from frame to frame
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UiFrameOutput {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u32>,
    pub draws: Vec<UiDraw>,
    // apply before drawing
    pub textures: Vec<UiTextureDelta>,
    pub texture_bytes: Vec<u8>,
    // free after drawing
    pub freed: Vec<u64>,
    pub actions: Vec<UiAction>,
    pub wants_pointer: bool,
    pub cursor: UiCursor,
    // the frame had too many vertices and was not written; texture changes are still present
    pub skipped: bool,
}

impl UiFrameOutput {
    pub fn new() -> Self {
        Self::default()
    }

    // empties every buffer and keeps its capacity
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.draws.clear();
        self.textures.clear();
        self.texture_bytes.clear();
        self.freed.clear();
        self.actions.clear();
        self.wants_pointer = false;
        self.cursor = UiCursor::Default;
        self.skipped = false;
    }
}
