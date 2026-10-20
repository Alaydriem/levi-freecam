use freecam_core::structs::RayRequest;

use crate::types::FcVec3;

// a ray for the shim to cast this frame; the other fields are meaningful only when `active` is 1
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcRayRequest {
    /// 1 when the shim casts the ray and answers with fc_ray_result before the second next fc_frame
    pub active: u8,
    pub origin: FcVec3,
    /// unit vector
    pub direction: FcVec3,
    /// blocks
    pub max_distance: f32,
}

impl From<Option<&RayRequest>> for FcRayRequest {
    fn from(value: Option<&RayRequest>) -> Self {
        value.map_or_else(Self::default, |ray| Self {
            active: 1,
            origin: FcVec3::from(&ray.origin),
            direction: FcVec3::from(&ray.direction),
            max_distance: ray.max_distance,
        })
    }
}
