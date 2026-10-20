use freecam_core::structs::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl FcVec3 {
    pub fn to_core(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl From<&Vec3> for FcVec3 {
    fn from(value: &Vec3) -> Self {
        Self { x: value.x, y: value.y, z: value.z }
    }
}
