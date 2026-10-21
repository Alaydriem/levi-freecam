use freecam_core::structs::{CameraCommand, FrameOutput};

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
    /// 1 while freecam controls the camera, until the controller is fully off
    pub camera_owned: u8,
    /// the marker changes to deliver this frame, in order; borrowed from the handle and valid until the next
    /// fc_frame or fc_destroy. Null when marker_count is 0
    pub markers: *const FcMarkerEntry,
    pub marker_count: usize,
    pub ray: FcRayRequest,
}

impl Default for FcFrameOutput {
    fn default() -> Self {
        Self {
            camera: 0,
            pose: FcPose::default(),
            has_override: 0,
            override_pose: FcPose::default(),
            camera_owned: 0,
            markers: std::ptr::null(),
            marker_count: 0,
            ray: FcRayRequest::default(),
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
        Self {
            camera,
            pose,
            has_override: u8::from(value.render_override.is_some()),
            override_pose: value.render_override.as_ref().map(FcPose::from).unwrap_or_default(),
            camera_owned: u8::from(value.camera_owned),
            markers: std::ptr::null(),
            marker_count: 0,
            ray: FcRayRequest::from(value.ray.as_ref()),
        }
    }
}
