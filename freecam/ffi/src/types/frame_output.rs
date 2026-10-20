use freecam_core::structs::{CameraCommand, FrameOutput, GuiCommand, MouseCommand};

use crate::types::{FcMarkerEntry, FcPose, FcRayRequest};

// what the shim does this frame; pose is meaningful only when camera is 1 and override_pose only when
// has_override is 1
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FcFrameOutput {
    // 0 none, 1 set, 2 clear
    pub camera: u8,
    pub pose: FcPose,
    pub has_override: u8,
    pub override_pose: FcPose,
    // 0 none, 1 hide, 2 show
    pub gui: u8,
    pub block_input: u8,
    /// 0 none, 1 release the mouse to the user, 2 grab it back for the game, 3 grab it for a look drag and keep the
    /// cursor position, 4 release it after a look drag and put the cursor back at the kept position
    pub mouse: u8,
    /// 1 while the UI is drawn; fc_ui_frame is called only then
    pub ui_visible: u8,
    /// 1 while the cursor is free and UI input goes to fc_ui_event
    pub cursor_mode: u8,
    /// 1 while a look drag holds the mouse in bar mode: the game may grab the mouse, the cursor stays hidden, and
    /// pointer moves do not go to fc_ui_event
    pub look_drag: u8,
    /// 1 while freecam controls the camera, until the controller is fully off
    pub camera_owned: u8,
    /// the marker changes to deliver this frame, in order; borrowed from the handle and valid until the next
    /// fc_frame or fc_destroy. Null when marker_count is 0
    pub markers: *const FcMarkerEntry,
    pub marker_count: usize,
    pub ray: FcRayRequest,
    /// 1 while a screenshot is under way: do not draw the UI, so the capture is clean; core removes the markers itself
    pub hide_ui: u8,
    /// 1 when the frame presented now must be captured; answer with fc_capture_taken and capture_sequence
    pub capture_active: u8,
    /// the request to echo in fc_capture_taken and fc_capture_written; meaningful only when capture_active is 1
    pub capture_sequence: u32,
}

impl Default for FcFrameOutput {
    fn default() -> Self {
        Self {
            camera: 0,
            pose: FcPose::default(),
            has_override: 0,
            override_pose: FcPose::default(),
            gui: 0,
            block_input: 0,
            mouse: 0,
            ui_visible: 0,
            cursor_mode: 0,
            look_drag: 0,
            camera_owned: 0,
            markers: std::ptr::null(),
            marker_count: 0,
            ray: FcRayRequest::default(),
            hide_ui: 0,
            capture_active: 0,
            capture_sequence: 0,
        }
    }
}

impl From<&FrameOutput> for FcFrameOutput {
    fn from(value: &FrameOutput) -> Self {
        let (camera, pose) = match &value.camera {
            CameraCommand::None => (0, FcPose::default()),
            CameraCommand::Set(pose) => (1, FcPose::from(pose)),
            CameraCommand::Clear => (2, FcPose::default()),
        };
        let gui = match value.gui {
            GuiCommand::None => 0,
            GuiCommand::Hide => 1,
            GuiCommand::Show => 2,
        };
        let mouse = match value.mouse {
            MouseCommand::None => 0,
            MouseCommand::Release => 1,
            MouseCommand::Grab => 2,
            MouseCommand::DragGrab => 3,
            MouseCommand::DragRelease => 4,
        };
        Self {
            camera,
            pose,
            has_override: u8::from(value.render_override.is_some()),
            override_pose: value.render_override.as_ref().map(FcPose::from).unwrap_or_default(),
            gui,
            block_input: u8::from(value.block_input),
            mouse,
            ui_visible: u8::from(value.ui_visible),
            cursor_mode: u8::from(value.cursor_mode),
            look_drag: u8::from(value.look_drag),
            camera_owned: u8::from(value.camera_owned),
            markers: std::ptr::null(),
            marker_count: 0,
            ray: FcRayRequest::from(value.ray.as_ref()),
            hide_ui: u8::from(value.hide_ui),
            capture_active: u8::from(value.capture.is_some()),
            capture_sequence: value.capture.map_or(0, |capture| capture.sequence),
        }
    }
}
