use freecam_core::structs::{CameraCommand, FrameOutput, GuiCommand};

use crate::types::FcPose;

// what the shim does this frame; pose is meaningful only when camera is 1 and override_pose only when
// has_override is 1, and show_overlay 1 means the shim draws the path overlay
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcFrameOutput {
    // 0 none, 1 set, 2 clear
    pub camera: u8,
    pub pose: FcPose,
    pub has_override: u8,
    pub override_pose: FcPose,
    // 0 none, 1 hide, 2 show
    pub gui: u8,
    pub block_input: u8,
    pub show_overlay: u8,
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
        Self {
            camera,
            pose,
            has_override: u8::from(value.render_override.is_some()),
            override_pose: value.render_override.as_ref().map(FcPose::from).unwrap_or_default(),
            gui,
            block_input: u8::from(value.block_input),
            show_overlay: u8::from(value.show_overlay),
        }
    }
}
