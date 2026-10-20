use crate::structs::{CameraPose, Vec3};

// the game state the shim reads once per frame
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameStatus {
    pub time_seconds: f64,
    pub in_world: bool,
    pub menu_open: bool,
    pub focused: bool,
    // the camera handler is captured and `minecraft:free` is in the preset list
    pub ready: bool,
    // the active camera is first person, third person or third person front
    pub player_camera_active: bool,
    // the player's head position and rotation
    pub start_pose: CameraPose,
    pub gui_hidden: bool,
    pub health: i32,
    pub dimension: i32,
    pub rendered_position: Option<Vec3>,
    pub game_camera_forward: Option<Vec3>,
    pub game_camera_position: Option<Vec3>,
    // vertical field of view of the game camera, degrees
    pub fov_y_degrees: f32,
    // width over height of the viewport
    pub aspect: f32,
    // viewport size in pixels
    pub screen_width: f32,
    pub screen_height: f32,
}
