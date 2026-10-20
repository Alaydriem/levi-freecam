use crate::structs::CameraPose;
use crate::structs::frame::{CameraCommand, GuiCommand};

// what the shim does this frame
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FrameOutput {
    pub camera: CameraCommand,
    pub render_override: Option<CameraPose>,
    pub gui: GuiCommand,
    pub block_input: bool,
    // true while the path editing overlay should be drawn
    pub show_overlay: bool,
}
