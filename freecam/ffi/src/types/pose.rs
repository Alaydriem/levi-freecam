use freecam_core::structs::CameraPose;

use crate::types::FcVec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcPose {
    pub position: FcVec3,
    // degrees, positive turns right
    pub yaw: f32,
    // degrees, positive looks down
    pub pitch: f32,
}

impl FcPose {
    pub fn to_core(&self) -> CameraPose {
        CameraPose::new(self.position.to_core(), self.yaw, self.pitch)
    }
}

impl From<&CameraPose> for FcPose {
    fn from(value: &CameraPose) -> Self {
        Self { position: FcVec3::from(&value.position), yaw: value.yaw, pitch: value.pitch }
    }
}
