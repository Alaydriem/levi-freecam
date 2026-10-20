use crate::structs::CameraPose;
use crate::structs::frame::{CameraCommand, CaptureRequest, GuiCommand, MouseCommand, RayRequest};

// what the shim does this frame
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FrameOutput {
    pub camera: CameraCommand,
    pub render_override: Option<CameraPose>,
    pub gui: GuiCommand,
    pub block_input: bool,
    pub mouse: MouseCommand,
    // true while the UI is drawn
    pub ui_visible: bool,
    pub cursor_mode: bool,
    // true while a right-button look drag in bar mode holds the mouse: the cursor stays hidden and the game may grab
    pub look_drag: bool,
    // true while freecam controls the camera: from turning on until fully off
    pub camera_owned: bool,
    // a ray to cast for a look-at key, answered with the controller's ray result
    pub ray: Option<RayRequest>,
    // true while a screenshot is under way: no mod drawing may reach the picture
    pub hide_ui: bool,
    // capture the frame presented now, answered with the controller's capture result
    pub capture: Option<CaptureRequest>,
}
