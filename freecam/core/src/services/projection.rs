use crate::structs::CameraPose;
use crate::structs::math::Vec3;

pub(crate) const NEAR_PLANE: f32 = 0.05;
const WORLD_UP: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };

// the view basis and the screen mapping for one build
pub(crate) struct Projection {
    origin: Vec3,
    right: Vec3,
    up: Vec3,
    forward: Vec3,
    focal: f32,
    aspect: f32,
    width: f32,
    height: f32,
}

impl Projection {
    pub(crate) fn new(view: CameraPose, fov_y_degrees: f32, aspect: f32, width: f32, height: f32) -> Self {
        let forward = view.forward();
        let right = forward.cross(WORLD_UP).normalized();
        Self {
            origin: view.position,
            right,
            up: right.cross(forward),
            forward,
            focal: 1.0 / (fov_y_degrees.to_radians() * 0.5).tan(),
            aspect,
            width,
            height,
        }
    }

    // the point relative to the camera as (right, up, forward)
    pub(crate) fn to_camera(&self, point: Vec3) -> Vec3 {
        let offset = point - self.origin;
        Vec3::new(offset.dot(self.right), offset.dot(self.up), offset.dot(self.forward))
    }

    // maps a camera space point in front of the near plane to pixels
    pub(crate) fn to_screen(&self, camera: Vec3) -> (f32, f32) {
        let x = self.width * 0.5 * (1.0 + self.focal * camera.x / (camera.z * self.aspect));
        let y = self.height * 0.5 * (1.0 - self.focal * camera.y / camera.z);
        (x, y)
    }

    // the part of the line in front of the near plane, or None when none of it is
    pub(crate) fn clip(from: Vec3, to: Vec3) -> Option<(Vec3, Vec3)> {
        let from_behind = from.z < NEAR_PLANE;
        let to_behind = to.z < NEAR_PLANE;
        match (from_behind, to_behind) {
            (true, true) => None,
            (false, false) => Some((from, to)),
            (true, false) => Some((Self::cut(from, to), to)),
            (false, true) => Some((from, Self::cut(to, from))),
        }
    }

    // the point where the line from `behind` to `ahead` crosses the near plane
    pub(crate) fn cut(behind: Vec3, ahead: Vec3) -> Vec3 {
        let fraction = (NEAR_PLANE - behind.z) / (ahead.z - behind.z);
        let at = behind + (ahead - behind) * fraction;
        Vec3::new(at.x, at.y, NEAR_PLANE)
    }
}
